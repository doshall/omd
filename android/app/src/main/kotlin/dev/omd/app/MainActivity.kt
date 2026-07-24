package dev.omd.app

import android.annotation.SuppressLint
import android.util.Base64
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.KeyEvent
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import dev.omd.app.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity() {

    private lateinit var binding: ActivityMainBinding
    private var filePathCallback: ValueCallback<Array<Uri>>? = null

    private val fileChooserLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        val callback = filePathCallback
        filePathCallback = null
        if (callback == null) return@registerForActivityResult

        val uris = WebChromeClient.FileChooserParams.parseResult(result.resultCode, result.data)
        callback.onReceiveValue(uris)
    }

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        setupWebView()
        setupBackNavigation()

        if (savedInstanceState == null) {
            binding.webView.loadUrl("file:///android_asset/index.html")
            handleIncomingIntent(intent)
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIncomingIntent(intent)
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun setupWebView() {
        binding.webView.apply {
            settings.apply {
                javaScriptEnabled = true
                domStorageEnabled = true
                allowFileAccess = true
                allowContentAccess = true
                useWideViewPort = true
                loadWithOverviewMode = true
                builtInZoomControls = false
                displayZoomControls = false
                cacheMode = WebSettings.LOAD_DEFAULT
                mixedContentMode = WebSettings.MIXED_CONTENT_COMPATIBILITY_MODE
                mediaPlaybackRequiresUserGesture = false
            }

            webViewClient = object : WebViewClient() {
                override fun shouldOverrideUrlLoading(
                    view: WebView?,
                    request: WebResourceRequest?
                ): Boolean {
                    val url = request?.url?.toString() ?: return false
                    // Stay in app for local assets
                    if (url.startsWith("file:///android_asset")) return false
                    // Open external links in browser
                    if (url.startsWith("http://") || url.startsWith("https://")) {
                        startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                        return true
                    }
                    return false
                }
            }

            webChromeClient = object : WebChromeClient() {
                override fun onShowFileChooser(
                    webView: WebView?,
                    filePathCallback: ValueCallback<Array<Uri>>?,
                    fileChooserParams: FileChooserParams?
                ): Boolean {
                    this@MainActivity.filePathCallback?.onReceiveValue(null)
                    this@MainActivity.filePathCallback = filePathCallback

                    val intent = fileChooserParams?.createIntent() ?: Intent(Intent.ACTION_GET_CONTENT).apply {
                        type = "*/*"
                        addCategory(Intent.CATEGORY_OPENABLE)
                    }
                    try {
                        fileChooserLauncher.launch(intent)
                    } catch (e: Exception) {
                        this@MainActivity.filePathCallback = null
                        Toast.makeText(this@MainActivity, R.string.file_chooser_error, Toast.LENGTH_SHORT).show()
                        return false
                    }
                    return true
                }
            }

            // Improve scroll performance on Android
            isVerticalScrollBarEnabled = true
            isHorizontalScrollBarEnabled = false
            overScrollMode = WebView.OVER_SCROLL_IF_CONTENT_SCROLLS
        }
    }

    private fun setupBackNavigation() {
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                binding.webView.evaluateJavascript(
                    "(function(){ return window.omdHasUnsavedChanges ? '1' : '0'; })();"
                ) { value ->
                    if (value == "\"1\"") {
                        showDiscardDialog {
                            finish()
                        }
                    } else if (binding.webView.canGoBack()) {
                        binding.webView.goBack()
                    } else {
                        isEnabled = false
                        onBackPressedDispatcher.onBackPressed()
                    }
                }
            }
        })
    }

    private fun showDiscardDialog(onDiscard: () -> Unit) {
        androidx.appcompat.app.AlertDialog.Builder(this)
            .setTitle(R.string.unsaved_title)
            .setMessage(R.string.unsaved_message)
            .setPositiveButton(R.string.unsaved_discard) { _, _ -> onDiscard() }
            .setNegativeButton(android.R.string.cancel, null)
            .show()
    }

    private fun handleIncomingIntent(intent: Intent?) {
        val uri = intent?.data ?: return
        if (intent.action != Intent.ACTION_VIEW) return

        try {
            contentResolver.openInputStream(uri)?.use { stream ->
                val text = stream.bufferedReader().readText()
                injectMarkdown(text, uri.lastPathSegment ?: "document.md")
            }
        } catch (e: Exception) {
            Toast.makeText(this, R.string.open_file_error, Toast.LENGTH_SHORT).show()
        }
    }

    private fun injectMarkdown(content: String, filename: String) {
        val b64 = Base64.encodeToString(content.toByteArray(Charsets.UTF_8), Base64.NO_WRAP)
        val escapedName = filename.replace("'", "\\'")

        binding.webView.evaluateJavascript(
            """
            (function() {
                var text = atob('$b64');
                localStorage.setItem('omd-web-content', text);
                localStorage.setItem('omd-web-filename', '$escapedName');
                location.reload();
            })();
            """.trimIndent(),
            null
        )
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        binding.webView.saveState(outState)
    }

    override fun onRestoreInstanceState(savedInstanceState: Bundle) {
        super.onRestoreInstanceState(savedInstanceState)
        binding.webView.restoreState(savedInstanceState)
    }

    override fun onPause() {
        super.onPause()
        binding.webView.onPause()
    }

    override fun onResume() {
        super.onResume()
        binding.webView.onResume()
    }

    override fun onDestroy() {
        binding.webView.destroy()
        super.onDestroy()
    }

    @Deprecated("Deprecated in Java")
    override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        if (keyCode == KeyEvent.KEYCODE_BACK && binding.webView.canGoBack()) {
            binding.webView.goBack()
            return true
        }
        return super.onKeyDown(keyCode, event)
    }
}
