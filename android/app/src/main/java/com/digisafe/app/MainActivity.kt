package com.digisafe.app

import android.graphics.fonts.FontStyle
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.AlertDialog
import androidx.compose.material.Button
import androidx.compose.material.Icon
import androidx.compose.material.IconButton
import androidx.compose.material.MaterialTheme
import androidx.compose.material.OutlinedTextField
import androidx.compose.material.Text
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.livedata.observeAsState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import com.digisafe.app.ui.theme.DigiSafeTheme
import com.lambdapioneer.argon2kt.Argon2Kt
import com.lambdapioneer.argon2kt.Argon2KtResult
import com.lambdapioneer.argon2kt.Argon2Mode
import java.security.MessageDigest
import javax.crypto.Cipher
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

class MainActivity : ComponentActivity() {
    companion object {
        init {
            System.loadLibrary("droid")
        }
    }
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            DigiSafeTheme {
                MakeUI()
            }
        }
    }
}


@Composable
fun MakeUI() {
    MainScreen()
    UnlockDialog()
}


class DigiSafeViewModel : ViewModel() {

    private val _key = MutableLiveData("")
    val key = _key
    private val _value = MutableLiveData("")
    val value = _value
    private val _isLocked = MutableLiveData(true)
    val isLocked = _isLocked
    private val _dbId = MutableLiveData("")
    val dbId = _dbId
    private val _rawPassword = MutableLiveData("")
    val rawPassword = _rawPassword

    private val dbMap = HashMap<String, String>()
    private var password = ""

    fun onKeyChange(newKey: String) {
        if (newKey.length <= 32) {
            _key.value = newKey
        }
    }

    fun onValueChange(newValue: String) {
        if (newValue.length <= 8000) {
            _value.value = newValue
        }
    }

    fun onUnlock() {
        val rawPasswordArray = _rawPassword.value?.toByteArray()
        _rawPassword.value = ""
        if (rawPasswordArray != null) {
            val argon2Kt = Argon2Kt()
            val hashResult: Argon2KtResult = argon2Kt.hash(
                mode = Argon2Mode.ARGON2_ID,
                password = rawPasswordArray,
                salt = "digisafe".toByteArray(),
                tCostInIterations = 2,
                mCostInKibibyte = 1048576,
            )
            val passwordHash = hashResult.encodedOutputAsString()
            password = passwordHash
            dbMap["_password"] = passwordHash
            _isLocked.value = false
        }
    }

    fun onDbIdChange(newDbId: String) {
        if (newDbId.length <= 8) {
            _dbId.value = newDbId
        }
    }

    fun onRawPasswordChange(newRawPassword: String) {
        if (newRawPassword.length <= 64) {
            _rawPassword.value = newRawPassword
        }
    }

    fun onGet() {
        if (_key.value !== null) {
            val dbValue = dbMap[_key.value]
            if (dbValue !== null) {
                _value.value = dbValue
            } else {
                _value.value = ""
            }
        }
    }

    fun onSet() {
        val kv = _key.value
        val vv = _value.value
        if (kv !== null && vv !== null) {
            dbMap[kv] = vv
        }
    }

}


@Composable
fun UnlockDialog(vm: DigiSafeViewModel = viewModel()) {

    val isLocked by vm.isLocked.observeAsState(initial = true)
    val dbId by vm.dbId.observeAsState(initial = "")
    val rawPassword by vm.rawPassword.observeAsState(initial = "")

    Column {
        if (isLocked) {

            val passwordVisible = remember { mutableStateOf(false) }

            AlertDialog(
                onDismissRequest = {},
                title = {
                    Text(text = "Unlock DigiSafe")
                },
                text = {
                    Column(verticalArrangement = Arrangement.Center) {
                        OutlinedTextField(
                            value = dbId,
                            onValueChange = { vm.onDbIdChange(it) },
                            label = { Text(text="Database Id") },
                            modifier = Modifier
                                .padding(top = 16.dp)
                                .sizeIn(minHeight = 1.dp)
                                .background(Color.Transparent)
                        )
                        OutlinedTextField(
                            value = rawPassword,
                            onValueChange = { vm.onRawPasswordChange(it) },
                            label = { Text(text="Password") },
                            visualTransformation = if (passwordVisible.value) VisualTransformation.None else PasswordVisualTransformation(),
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                            trailingIcon = {
                                val image = if (passwordVisible.value)
                                    Icons.Filled.Visibility
                                else
                                    Icons.Filled.VisibilityOff
                                val description = if (passwordVisible.value) "Hide password" else "Show password"
                                IconButton(onClick = {passwordVisible.value = !passwordVisible.value}) {
                                    Icon(imageVector = image, description)
                                }
                            },
                            modifier = Modifier
                                .background(Color.Transparent)
                                .padding(top = 16.dp)
                                .sizeIn(minHeight = 1.dp)
                        )
                    }
                },
                confirmButton = {
                    Button(onClick = { vm.onUnlock() }) {
                        Text(
                            "Unlock",
                            style = TextStyle(
                                background = MaterialTheme.colors.primary,
                                color = MaterialTheme.colors.onPrimary,
                                fontSize = 18.sp,
                                fontWeight = FontWeight(FontStyle.FONT_WEIGHT_BOLD),
                            )
                        )
                    }
                },
                dismissButton = {}
            )
        }
    }
}


@Composable
fun MainScreen(vm: DigiSafeViewModel = viewModel()) {

    val key by vm.key.observeAsState(initial = "")
    val value by vm.value.observeAsState(initial = "")
    
    Box(
        modifier = Modifier
            .background(color = MaterialTheme.colors.background)
            .fillMaxSize()
    ) {
        Column(
            verticalArrangement = Arrangement.Center,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Row (
                horizontalArrangement = Arrangement.Center,
                modifier = Modifier.fillMaxWidth()
            ) {
                OutlinedTextField(
                    value = key,
                    onValueChange = { vm.onKeyChange(it) },
                    label = { Text(text = "Key") },
                    modifier = Modifier
                        .background(Color.Transparent)
                        .fillMaxWidth()
                        .padding(top = 16.dp)
                        .sizeIn(minHeight = 1.dp)
                )
            }
            Row (
                horizontalArrangement = Arrangement.Center,
                modifier = Modifier.fillMaxWidth()
            ) {
                OutlinedTextField(
                    value = value,
                    onValueChange = { vm.onValueChange(it) },
                    label = { Text(text = "Value") },
                    modifier = Modifier
                        .background(Color.Transparent)
                        .fillMaxHeight(0.52F)
                        .fillMaxWidth()
                        .padding(top = 16.dp, bottom = 16.dp)
                )
            }
            Row (
                horizontalArrangement = Arrangement.Center,
                modifier = Modifier.fillMaxWidth()
            ) {
                Button(onClick = { vm.onGet() }) {
                    Text(
                        "Get",
                        style = TextStyle(
                            color = MaterialTheme.colors.onPrimary,
                            background = MaterialTheme.colors.primary,
                            fontSize = 18.sp,
                            fontWeight = FontWeight(FontStyle.FONT_WEIGHT_BOLD),
                        ),
                        modifier = Modifier.padding(horizontal = 16.dp)
                    )
                }
                Spacer(modifier = Modifier.width(48.dp))
                Button(onClick = { vm.onSet() }) {
                    Text(
                        "Set",
                        style = TextStyle(
                            color = MaterialTheme.colors.onPrimary,
                            background = MaterialTheme.colors.primary,
                            fontSize = 18.sp,
                            fontWeight = FontWeight(FontStyle.FONT_WEIGHT_BOLD),
                        ),
                        modifier = Modifier.padding(horizontal = 16.dp)
                    )
                }
                Spacer(modifier = Modifier.width(48.dp))
                Button(onClick = {}) {
                    Text(
                        "Save",
                        style = TextStyle(
                            color = MaterialTheme.colors.onPrimary,
                            background = MaterialTheme.colors.primary,
                            fontSize = 18.sp,
                            fontWeight = FontWeight(FontStyle.FONT_WEIGHT_BOLD),
                        ),
                        modifier = Modifier.padding(horizontal = 10.dp)
                    )
                }
            }
        }
    }
}
