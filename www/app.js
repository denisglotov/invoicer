import init, {
  init_panic_hook,
  parse_pdf_and_create_payment,
  parse_text_and_create_payment
} from './pkg/invoicer.js';

// DOM elements
const uploadView = document.getElementById('upload-view');
const loadingView = document.getElementById('loading-view');
const resultView = document.getElementById('result-view');

const dropzone = document.getElementById('dropzone');
const fileInput = document.getElementById('file-input');
const btnParseText = document.getElementById('btn-parse-text');
const pasteText = document.getElementById('paste-text');
const btnReset = document.getElementById('btn-reset');
const btnInstall = document.getElementById('btn-install');
const toast = document.getElementById('toast');

// Result fields
const resSupplier = document.getElementById('res-supplier');
const resRecipient = document.getElementById('res-recipient');
const resAmount = document.getElementById('res-amount');
const resDueDate = document.getElementById('res-due-date');
const resQrSvg = document.getElementById('res-qr-svg');
const resIban = document.getElementById('res-iban');
const resReference = document.getElementById('res-reference');
const resAmountFormatted = document.getElementById('res-amount-formatted');
const resAmountRaw = document.getElementById('res-amount-raw');
const resBic = document.getElementById('res-bic');
const resInvoiceNum = document.getElementById('res-invoice-num');

// Action buttons
const btnDownloadQr = document.getElementById('btn-download-qr');
const btnCopyAll = document.getElementById('btn-copy-all');

let currentPaymentResult = null;
let deferredPrompt = null;
let wasmReady = false;

// 1. Initialize WebAssembly & Service Worker
async function bootstrap() {
  try {
    await init();
    init_panic_hook();
    wasmReady = true;
    console.log('Rust WebAssembly engine initialized successfully.');

    // Register Service Worker for PWA & Share Target
    if ('serviceWorker' in navigator) {
      try {
        await navigator.serviceWorker.register('./sw.js');
        console.log('PWA Service Worker registered.');
      } catch (err) {
        console.warn('Service Worker registration skipped:', err);
      }
    }

    // Check if opened via Web Share Target
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get('shared') === '1') {
      await handleSharedInvoice();
    }
  } catch (err) {
    console.error('Failed to initialize Invoicer:', err);
    showToast('Failed to load Wasm engine: ' + err.message);
  }
}

// 2. Handle shared PDF from Android Share Target via CacheStorage
async function handleSharedInvoice() {
  try {
    if ('caches' in window) {
      const cache = await caches.open('invoicer-shared-v1');
      const response = await cache.match('shared-invoice-pdf');
      if (response) {
        showView('loading');
        const blob = await response.blob();
        const buffer = await blob.arrayBuffer();
        await cache.delete('shared-invoice-pdf');
        // Clean URL parameter
        window.history.replaceState({}, document.title, window.location.pathname);
        await processPdfBytes(new Uint8Array(buffer));
      }
    }
  } catch (err) {
    console.error('Error reading shared invoice:', err);
    showToast('Could not process shared invoice');
    showView('upload');
  }
}

// 3. Process PDF bytes using Rust WebAssembly
async function processPdfBytes(uint8Array) {
  if (!wasmReady) {
    showToast('Engine initializing, please wait...');
    return;
  }

  showView('loading');

  try {
    const paymentResult = parse_pdf_and_create_payment(uint8Array);
    renderPaymentResult(paymentResult);
    showView('result');
  } catch (err) {
    console.error('Parsing error:', err);
    showToast('Error: ' + err);
    showView('upload');
  }
}

// 4. Process raw text using Rust WebAssembly
function processText(text) {
  if (!text || !text.trim()) {
    showToast('Please enter or paste invoice text');
    return;
  }

  showView('loading');
  try {
    const paymentResult = parse_text_and_create_payment(text);
    renderPaymentResult(paymentResult);
    showView('result');
  } catch (err) {
    console.error('Text parsing error:', err);
    showToast('Error: ' + err);
    showView('upload');
  }
}

// 5. Render parsed invoice and QR code in Result view
function renderPaymentResult(result) {
  currentPaymentResult = result;
  const inv = result.invoice;

  resSupplier.textContent = inv.supplier_name || 'Supplier';
  resRecipient.textContent = inv.recipient_name ? `Bill to: ${inv.recipient_name}` : '';
  resAmount.textContent = `€${inv.formatted_amount}`;
  resDueDate.textContent = inv.due_date ? `Due: ${inv.due_date}` : 'Due on receipt';

  // Inject SVG QR Code
  resQrSvg.innerHTML = result.qr_svg;

  // Populate transfer detail fields
  resIban.textContent = inv.iban || '-';
  resReference.textContent = inv.reference || '-';
  resAmountFormatted.textContent = `${inv.formatted_amount} EUR`;
  resAmountRaw.value = inv.formatted_amount;
  resBic.textContent = inv.bic || '-';
  resInvoiceNum.textContent = inv.invoice_num || '-';
}

// 6. View switcher
function showView(viewName) {
  uploadView.classList.toggle('active', viewName === 'upload');
  loadingView.classList.toggle('active', viewName === 'loading');
  resultView.classList.toggle('active', viewName === 'result');
}

// 7. Toast notification helper
function showToast(message) {
  toast.textContent = message;
  toast.classList.add('show');
  setTimeout(() => {
    toast.classList.remove('show');
  }, 2200);
}

// 8. Event Listeners

// File input
fileInput.addEventListener('change', async (e) => {
  const file = e.target.files[0];
  if (file) {
    const buffer = await file.arrayBuffer();
    await processPdfBytes(new Uint8Array(buffer));
  }
  fileInput.value = '';
});

// Drag and drop
dropzone.addEventListener('dragover', (e) => {
  e.preventDefault();
  dropzone.classList.add('dragover');
});

dropzone.addEventListener('dragleave', () => {
  dropzone.classList.remove('dragover');
});

dropzone.addEventListener('drop', async (e) => {
  e.preventDefault();
  dropzone.classList.remove('dragover');
  const file = e.dataTransfer.files[0];
  if (file) {
    const buffer = await file.arrayBuffer();
    await processPdfBytes(new Uint8Array(buffer));
  }
});

// Text parse button
btnParseText.addEventListener('click', () => {
  processText(pasteText.value);
});

// Reset button
btnReset.addEventListener('click', () => {
  showView('upload');
});

// Download / Save QR Code Image
if (btnDownloadQr) {
  btnDownloadQr.addEventListener('click', () => {
    if (!currentPaymentResult || !currentPaymentResult.qr_svg) {
      showToast('No QR code available');
      return;
    }
    const svgBlob = new Blob([currentPaymentResult.qr_svg], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(svgBlob);
    const downloadLink = document.createElement('a');
    const invNr = currentPaymentResult.invoice.invoice_num || 'invoice';
    downloadLink.href = url;
    downloadLink.download = `SEPA-QR-${invNr}.svg`;
    document.body.appendChild(downloadLink);
    downloadLink.click();
    document.body.removeChild(downloadLink);
    URL.revokeObjectURL(url);
    showToast('QR code saved to downloads!');
  });
}

// Copy All Transfer Details
if (btnCopyAll) {
  btnCopyAll.addEventListener('click', async () => {
    if (!currentPaymentResult) return;
    const inv = currentPaymentResult.invoice;
    const fullDetails = [
      `Beneficiary: ${inv.supplier_name}`,
      `IBAN: ${inv.iban}`,
      inv.bic ? `BIC: ${inv.bic}` : null,
      `Amount: ${inv.formatted_amount} EUR`,
      `Reference: ${inv.reference}`
    ].filter(Boolean).join('\n');

    try {
      await navigator.clipboard.writeText(fullDetails);
      showToast('All payment details copied!');
    } catch (err) {
      showToast('Failed to copy details');
    }
  });
}

// Individual field copy buttons
document.querySelectorAll('.btn-copy').forEach((button) => {
  button.addEventListener('click', async () => {
    const targetId = button.dataset.target;
    const el = document.getElementById(targetId);
    if (!el) return;

    const textToCopy = el.value || el.textContent;
    if (!textToCopy || textToCopy === '-') return;

    try {
      await navigator.clipboard.writeText(textToCopy.trim());
      button.classList.add('copied');
      showToast('Copied to clipboard!');
      setTimeout(() => button.classList.remove('copied'), 1500);
    } catch (err) {
      showToast('Failed to copy');
    }
  });
});

// PWA Install prompt
window.addEventListener('beforeinstallprompt', (e) => {
  e.preventDefault();
  deferredPrompt = e;
  btnInstall.classList.remove('hidden');
});

btnInstall.addEventListener('click', async () => {
  if (deferredPrompt) {
    deferredPrompt.prompt();
    const { outcome } = await deferredPrompt.userChoice;
    console.log(`User response to install: ${outcome}`);
    deferredPrompt = null;
    btnInstall.classList.add('hidden');
  }
});

// Initialize on page load
bootstrap();
