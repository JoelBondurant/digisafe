buildscript {
   val composeVersion by extra("1.1.1")
}
plugins {
    id("com.android.application") version "7.4.0" apply false
    id("com.android.library") version "7.4.0" apply false
    id("org.jetbrains.kotlin.android") version "1.6.10" apply false
}