import java.io.File
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.logging.LogLevel
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import javax.inject.Inject
import org.gradle.process.ExecOperations

private fun androidNdkBin(): File {
    val configuredNdk = listOf("NDK_HOME", "ANDROID_NDK_HOME", "ANDROID_NDK_ROOT")
        .firstNotNullOfOrNull { name -> System.getenv(name)?.takeIf { it.isNotBlank() } }
    val ndk = configuredNdk?.let(::File) ?: run {
        val sdk = listOf("ANDROID_HOME", "ANDROID_SDK_ROOT")
            .firstNotNullOfOrNull { name -> System.getenv(name)?.takeIf { it.isNotBlank() } }
        val installed = sdk?.let { File(it, "ndk") }
            ?.listFiles()
            ?.filter { it.isDirectory }
            ?.maxByOrNull { it.name }
        installed ?: throw GradleException("Set NDK_HOME or install an NDK under ANDROID_HOME/ndk")
    }
    val prebuilt = File(ndk, "toolchains/llvm/prebuilt")
    val os = System.getProperty("os.name").lowercase()
    val arch = System.getProperty("os.arch").lowercase()
    val preferredHosts = when {
        os.contains("windows") -> listOf("windows-x86_64")
        os.contains("mac") && (arch == "aarch64" || arch == "arm64") -> listOf("darwin-arm64", "darwin-x86_64")
        os.contains("mac") -> listOf("darwin-x86_64")
        arch == "aarch64" || arch == "arm64" -> listOf("linux-aarch64", "linux-x86_64")
        else -> listOf("linux-x86_64")
    }
    val host = (preferredHosts + prebuilt.listFiles().orEmpty().filter { it.isDirectory }.map { it.name })
        .distinct()
        .map { File(prebuilt, it) }
        .firstOrNull { it.isDirectory }
        ?: throw GradleException("No NDK LLVM toolchain found under ${prebuilt.path}")
    val bin = File(host, "bin")
    if (!bin.isDirectory) throw GradleException("NDK LLVM bin directory does not exist: ${bin.path}")
    return bin
}

abstract class BuildTask : DefaultTask() {
    @get:Inject
    abstract val execOperations: ExecOperations

    @Input
    var rootDirRel: String? = null
    @Input
    var projectDir: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null

    @TaskAction
    fun build() {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val projectDir = projectDir ?: throw GradleException("projectDir cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")
        val cargoRoot = File(File(projectDir), rootDirRel).canonicalFile
        if (!File(cargoRoot, "Cargo.toml").isFile) {
            throw GradleException("Cargo.toml was not found at ${cargoRoot.path}")
        }

        val (rustTarget, androidAbi) = when (target) {
            "aarch64" -> "aarch64-linux-android" to "arm64-v8a"
            "armv7" -> "armv7-linux-androideabi" to "armeabi-v7a"
            "i686" -> "i686-linux-android" to "x86"
            "x86_64" -> "x86_64-linux-android" to "x86_64"
            else -> throw GradleException("Unsupported Android Rust target: $target")
        }

        val ndkBin = androidNdkBin()
        val path = listOf(ndkBin.path, System.getenv("PATH").orEmpty())
            .filter { it.isNotEmpty() }
            .joinToString(File.pathSeparator)

        execOperations.exec {
            workingDir(cargoRoot)
            environment("PATH", path)
            executable("cargo")
            args("build", "--lib", "--target", rustTarget)
            if (project.logger.isEnabled(LogLevel.DEBUG)) {
                args("--verbose", "--verbose")
            } else if (project.logger.isEnabled(LogLevel.INFO)) {
                args("--verbose")
            }
            if (release) {
                args("--release")
            }
        }.assertNormalExitValue()

        val profile = if (release) "release" else "debug"
        val builtLibrary = File(cargoRoot, "target/$rustTarget/$profile/libzorness_android.so")
        if (!builtLibrary.isFile) {
            throw GradleException("Rust shared library was not produced at ${builtLibrary.path}")
        }
        val jniLibsDir = File(projectDir, "src/main/jniLibs/$androidAbi")
        if (!jniLibsDir.exists() && !jniLibsDir.mkdirs()) {
            throw GradleException("Could not create Android JNI library directory: ${jniLibsDir.path}")
        }
        builtLibrary.copyTo(File(jniLibsDir, builtLibrary.name), overwrite = true)
    }
}
