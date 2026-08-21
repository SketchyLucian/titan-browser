import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
}

val repositoryRoot = rootProject.projectDir.parentFile
val generatedWebAssets = layout.buildDirectory.dir("generated/web-assets")
val npmExecutable = if (System.getProperty("os.name").startsWith("Windows", ignoreCase = true)) {
    "npm.cmd"
} else {
    "npm"
}

fun releaseSecret(name: String): String? = providers.gradleProperty(name)
    .orElse(providers.environmentVariable(name))
    .orNull

val releaseStoreFile = releaseSecret("TITAN_RELEASE_STORE_FILE")
val releaseStorePassword = releaseSecret("TITAN_RELEASE_STORE_PASSWORD")
val releaseKeyAlias = releaseSecret("TITAN_RELEASE_KEY_ALIAS")
val releaseKeyPassword = releaseSecret("TITAN_RELEASE_KEY_PASSWORD")
val hasReleaseSigning = listOf(
    releaseStoreFile,
    releaseStorePassword,
    releaseKeyAlias,
    releaseKeyPassword
).all { !it.isNullOrBlank() }

val buildWebScripts by tasks.registering(Exec::class) {
    workingDir(repositoryRoot)
    commandLine(npmExecutable, "run", "build:web-scripts")
    inputs.dir(repositoryRoot.resolve("web-scripts/src"))
    inputs.file(repositoryRoot.resolve("web-scripts/tsconfig.json"))
    outputs.dir(repositoryRoot.resolve("web-scripts/dist"))
}

val syncAndroidWebScript by tasks.registering(Sync::class) {
    dependsOn(buildWebScripts)
    from(
        repositoryRoot.resolve("web-scripts/dist/android-adblock.js"),
        repositoryRoot.resolve("web-scripts/dist/android-privacy.js")
    )
    into(generatedWebAssets)
}

android {
    namespace = "com.titan.browser"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.titan.browser"
        minSdk = 26
        targetSdk = 36
        versionCode = 8
        versionName = "0.4.4"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }
    }

    signingConfigs {
        if (hasReleaseSigning) {
            create("release") {
                storeFile = file(requireNotNull(releaseStoreFile))
                storePassword = releaseStorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
                enableV1Signing = true
                enableV2Signing = true
                enableV3Signing = true
                enableV4Signing = true
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.findByName("release")
        }
        debug {
            applicationIdSuffix = ".debug"
            isDebuggable = true
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }

    lint {
        // Full debug and release lint are explicit CI gates. Avoid AGP's reduced
        // lintVital task, which can race its migrated lint-registry cache on Windows.
        checkReleaseBuilds = false
        // AAPT still requires adaptive-icon XML in a v26-qualified directory.
        disable += "ObsoleteSdkInt"
    }

    sourceSets.getByName("main").assets.srcDir(generatedWebAssets)
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

tasks.named("preBuild").configure {
    dependsOn(syncAndroidWebScript)
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.activity.compose)

    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.ui)
    implementation(libs.androidx.ui.graphics)
    implementation(libs.androidx.ui.tooling.preview)
    implementation(libs.androidx.material3)
    implementation(libs.androidx.material.icons.extended)

    implementation(libs.androidx.webkit)
    implementation(libs.kotlinx.coroutines.android)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.androidx.datastore.preferences)

    debugImplementation(libs.androidx.ui.tooling)
    testImplementation(libs.junit)
}
