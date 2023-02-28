package com.digisafe.app

fun ByteArray.toHexString() = joinToString(",") { it.toUByte().toString() }

external fun encrypt(
    key: ByteArray,
    nonce: ByteArray,
    arg: ByteArray): ByteArray

external fun sha3(
    arg: ByteArray): ByteArray
