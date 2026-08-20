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

val buildWebScripts by tasks.registering(Exec::class) {
    workingDir(repositoryRoot)
    commandLine(npmExecutable, "run", "build:web-scripts")
    inputs.dir(repositoryRoot.resolve("web-scripts/src"))
    inputs.file(repositoryRoot.resolve("web-scripts/tsconfig.json"))
    outputs.dir(repositoryRoot.resolve("web-scripts/dist"))
}

val syncAndroidWebScript by tasks.registering(Sync::class) {
    dependsOn(buildWebScripts)
    from(repositoryRoot.resolve("web-scripts/dist/android-adblock.js"))
    into(generatedWebAssets)
}

android {
    namespace = "com.titan.browser"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.titan.browser"
        minSdk = 26
        targetSdk = 35
        versionCode = 5
        versionName = "0.4.1"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug")
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

    kotlinOptions {
        jvmTarget = "17"
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

    sourceSets.getByName("main").assets.srcDir(generatedWebAssets)
}

tasks.named("preBuild").configure {
    dependsOn(syncAndroidWebScript)
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
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
