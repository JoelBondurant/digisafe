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
import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import com.digisafe.app.ui.theme.DigiSafeTheme


class MainActivity : ComponentActivity() {
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
    private val _value = MutableLiveData("")
    private val _isLocked = MutableLiveData(true)
    private val _dbId = MutableLiveData("")
    private val _rawPassword = MutableLiveData("")
    val key = _key
    val value = _value
    val isLocked = _isLocked
    val dbId = _dbId
    val rawPassword = _rawPassword
    val dbMap = HashMap<String, String>()
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
    fun onLockChange() {
        _isLocked.value = false
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
            val dbValue = dbMap.get(_key.value)
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
            dbMap.put(kv, vv)
        }
    }
}

@Composable
fun UnlockDialog(digiSafeViewModel: DigiSafeViewModel = viewModel()) {
    val isLocked by digiSafeViewModel.isLocked.observeAsState(initial = true)
    val dbId by digiSafeViewModel.dbId.observeAsState(initial = "")
    val rawPassword by digiSafeViewModel.rawPassword.observeAsState(initial = "")
    UnlockDialogContent(
        isLocked = isLocked,
        onLockChange = { digiSafeViewModel.onLockChange() },
        dbId = dbId,
        onDbIdChange = { digiSafeViewModel.onDbIdChange(it) },
        rawPassword = rawPassword,
        onRawPasswordChange = { digiSafeViewModel.onRawPasswordChange(it) },
    )
}

@Composable
fun UnlockDialogContent(
    isLocked: Boolean,
    onLockChange: () -> Unit,
    dbId: String,
    onDbIdChange: (String) -> Unit,
    rawPassword: String,
    onRawPasswordChange: (String) -> Unit,
) {
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
                            onValueChange = onDbIdChange,
                            label = { Text(text="Database Id") },
                            modifier = Modifier
                                .padding(top = 16.dp)
                                .sizeIn(minHeight = 1.dp)
                                .background(Color.Transparent)
                        )
                        OutlinedTextField(
                            value = rawPassword,
                            onValueChange = onRawPasswordChange,
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
                                .padding(top = 16.dp)
                                .sizeIn(minHeight = 1.dp)
                                .background(Color.Transparent)
                        )
                    }
                },
                confirmButton = {
                    Button(onClick = onLockChange) {
                        Text(
                            "Unlock",
                            style = TextStyle(
                                color = MaterialTheme.colors.onPrimary,
                                background = MaterialTheme.colors.primary,
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
fun MainScreen(digiSafeViewModel: DigiSafeViewModel = viewModel()) {
    val key by digiSafeViewModel.key.observeAsState(initial = "")
    val value by digiSafeViewModel.value.observeAsState(initial = "")
    MainContent(
        key = key,
        onKeyChange = { digiSafeViewModel.onKeyChange(it) },
        value = value,
        onValueChange = { digiSafeViewModel.onValueChange(it) },
        onGet = { digiSafeViewModel.onGet() },
        onSet = { digiSafeViewModel.onSet() },
    )
}

@Composable
fun MainContent(
    key: String,
    onKeyChange: (String) -> Unit,
    value: String,
    onValueChange: (String) -> Unit,
    onGet: () -> Unit,
    onSet: () -> Unit,
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(color = MaterialTheme.colors.background)
    ) {
        Column(
            verticalArrangement = Arrangement.Center,
            modifier = Modifier.fillMaxWidth(),
        ) {
            OutlinedTextField(
                value = key,
                onValueChange = onKeyChange,
                label = { Text(text = "Key") },
                modifier = Modifier
                    .padding(top = 16.dp)
                    .sizeIn(minHeight = 1.dp)
                    .background(Color.Transparent)
                    .fillMaxWidth()
            )
            OutlinedTextField(
                value = value,
                onValueChange = onValueChange,
                label = { Text(text = "Value") },
                modifier = Modifier
                    .padding(top = 16.dp)
                    .sizeIn(minHeight = 400.dp)
                    .background(Color.Transparent)
                    .fillMaxWidth()
            )
            Row (
                horizontalArrangement = Arrangement.Center,
                modifier = Modifier.fillMaxWidth()
            ) {
                Button(onClick = onGet) {
                    Text(
                        "Get",
                        style = TextStyle(
                            color = MaterialTheme.colors.onPrimary,
                            background = MaterialTheme.colors.primary,
                            fontWeight = FontWeight(FontStyle.FONT_WEIGHT_BOLD),
                        ),
                        modifier = Modifier.padding(horizontal = 16.dp)
                    )
                }
                Spacer(modifier = Modifier.width(48.dp))
                Button(onClick = onSet) {
                    Text(
                        "Set",
                        style = TextStyle(
                            color = MaterialTheme.colors.onPrimary,
                            background = MaterialTheme.colors.primary,
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
                            fontWeight = FontWeight(FontStyle.FONT_WEIGHT_BOLD),
                        ),
                        modifier = Modifier.padding(horizontal = 10.dp)
                    )
                }
            }
        }
    }
}
