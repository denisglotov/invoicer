# Invoicer 🧾⚡

A privacy-first, offline **Progressive Web App (PWA)** built with a pure **Rust WebAssembly (Wasm)** engine. It parses PDF utility invoices on mobile and desktop, automatically extracting payment details and generating a **SEPA EPC QR Code** and **Revolut transfer link** for instant one-tap bill payments.

## 🌟 Key Features

* **⚡ 100% Client-Side WebAssembly**: Powered by Rust compiled to WebAssembly via `wasm-bindgen`. All PDF parsing and QR generation happen completely inside the browser runtime. Zero invoice data leaves your device.
* **📱 Android Share Target**: Installable as a PWA on Android. Share any invoice PDF directly from **Gmail** or your file manager straight to Invoicer!
* **📲 iOS & Desktop Support**: Tap or drag-and-drop any PDF invoice to parse instantly on iPhone, iPad, Mac, and PC.
* **🏧 Standard SEPA EPC QR Code (BCD v002)**: Generates a crisp vector QR code compatible with Revolut, Swedbank, SEB, Luminor, and all European banking apps.
* **💳 Revolut One-Tap Actions**:
  * "Open Revolut App" direct action button
  * One-tap copy buttons for **IBAN**, **Payment Reference / Purpose**, **Amount**, and **BIC** with haptic/visual feedback.
* **🧠 Smart Multi-Template Parser**:
  * Specialized extractor for **OneBaltic Property Management SIA** invoices (extracts exact invoice number, OBI barcode reference `OBI/57559/127.43`, Industra Bank IBAN `LV97MULT1010A80170010`, BIC `MULTLV2X`, and total amount `Kopā apmaksai`).
  * Generic fallback heuristic parser for standard European invoices with IBAN, BIC, and EUR amounts.

## 🏗️ Architecture

```
                       ┌───────────────────────────┐
                       │   Gmail / Mobile Share    │
                       │   (PDF Invoice shared)    │
                       └─────────────┬─────────────┘
                                     │ (Android Web Share Target / iOS File Picker)
                                     ▼
┌────────────────────────────────────────────────────────────────────────────┐
│ Progressive Web App (PWA Shell)                                            │
│                                                                            │
│  ┌──────────────────────┐                  ┌────────────────────────────┐  │
│  │ Service Worker       │ (Intercepts file │ Responsive Mobile UI       │  │
│  │ (sw.js & manifest)   │  POST share)     │ (HTML5 / Modern CSS / JS)  │  │
│  └──────────┬───────────┘                  └──────────────┬─────────────┘  │
│             │                                             │                │
│             ▼                                             │                │
│  ┌────────────────────────────────────────────────────────┴─────────────┐  │
│  │ Rust WebAssembly Core (`www/pkg/invoicer.wasm`)                      │  │
│  │                                                                      │  │
│  │  1. PDF Text Extractor (`lopdf` + operator stream walker)            │  │
│  │  2. Invoice Rule Parsers (`OneBaltic` & Generic heuristic)           │  │
│  │  3. SEPA EPC QR Code Generator (`qrcode` SVG renderer)               │  │
│  │  4. Revolut Transfer Payload Builder                                 │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                    │                                       │
│                                    ▼                                       │
│                 ┌─────────────────────────────────────┐                    │
│                 │  Direct Quick-Pay Result View       │                    │
│                 │  - SEPA EPC QR Code (Revolut Scan)  │                    │
│                 │  - "Open Revolut App" Deep Link     │                    │
│                 │  - One-Tap Copy: IBAN, Amt, Ref     │                    │
│                 └─────────────────────────────────────┘                    │
└────────────────────────────────────────────────────────────────────────────┘
```

## 🚀 Getting Started

### Prerequisites

* [Rust](https://rustup.rs/) (with `wasm32-unknown-unknown` target):
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
* `wasm-bindgen-cli`:
  ```bash
  cargo install wasm-bindgen-cli --version 0.2.127
  ```
* [`just`](https://github.com/casey/just) command runner (optional, or run `./build.sh`)

### 🔨 Building the Project

Run with `just`:

```bash
just build
```

*Or via the shell script:*
```bash
./build.sh
```

### 🧪 Running Tests & Linting

Run the test suite:

```bash
just test
```

Run linter (`clippy` on native and wasm targets + `rustfmt` check):

```bash
just lint
```

Format codebase:

```bash
just fmt
```

### 🌐 Running Locally

Start a local static development web server:

```bash
just serve
```
*Or specify a custom port:*
```bash
just serve 8080
```

Open `http://localhost:3000` in your browser.

## 📲 How to Use on Your Phone

### On Android (with Gmail Share Target):
1. Open the hosted web app URL (HTTPS required for PWA service workers).
2. Tap the **Install** button in the header (or browser menu -> **"Add to Home Screen"**).
3. When you receive an invoice in **Gmail**, tap the attachment's **Share** icon and select **Invoicer**.
4. Invoicer opens instantly, parses the bill, and displays the SEPA EPC QR code with one-tap copy buttons and Revolut links.

### On iPhone (iOS):
1. Open the app in Safari and tap **Share -> Add to Home Screen**.
2. Open the app and tap **"Choose Invoice PDF"** to pick your invoice.

