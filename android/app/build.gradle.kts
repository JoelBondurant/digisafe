plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}
android {
    namespace = "com.digisafe.app"
    compileSdk = 32

    defaultConfig {
        applicationId = "com.digisafe.app"
        minSdk = 30
        targetSdk = 32
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        compose = true
    }
    composeOptions {
        kotlinCompilerExtensionVersion = rootProject.extra["composeVersion"] as String
    }
    packagingOptions {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
}

dependencies {

    androidTestImplementation("androidx.compose.ui:ui-test-junit4:${rootProject.extra["composeVersion"]}")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    debugImplementation("androidx.compose.ui:ui-test-manifest:${rootProject.extra["composeVersion"]}")
    debugImplementation("androidx.compose.ui:ui-tooling:${rootProject.extra["composeVersion"]}")
    implementation("androidx.activity:activity-compose:1.3.1")
    implementation("androidx.compose.material:material-icons-extended:${rootProject.extra["composeVersion"]}")
    implementation("androidx.compose.material:material:${rootProject.extra["composeVersion"]}")
    implementation("androidx.compose.runtime:runtime-livedata:${rootProject.extra["composeVersion"]}")
    implementation("androidx.compose.ui:ui-tooling-preview:${rootProject.extra["composeVersion"]}")
    implementation("androidx.compose.ui:ui:${rootProject.extra["composeVersion"]}")
    implementation("androidx.core:core-ktx:1.7.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.3.1")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.5.1")
    implementation("com.lambdapioneer.argon2kt:argon2kt:1.3.0")
    testImplementation("junit:junit:4.13.2")

}