# Budgy — Local-First Budget Tracker

**Date:** 2026-04-06
**Status:** Draft for review

---

## 1. Concept & Vision

Budgy is a privacy-first mobile budget tracker that runs entirely offline — no cloud, no developer access to user data. The app uses a local LLM (Qwen 2.5 3B) to scan receipts, categorize expenses, detect recurring bills, and provide personalized financial advice via an RAG-powered knowledge base. The standout feature is **Projects** — allowing users to create separate budget buckets for specific goals or events with their own categories and tracking.

**Core promise:** Your financial data stays on your device.

---

## 2. Architecture

### Stack
- **Frontend:** Tauri WebView (HTML/CSS/JS)
- **Backend:** Rust
- **LLM:** Qwen 2.5 3B (quantized GGUF)
- **Database:** SQLite
- **Image storage:** Local file system

### Data Flow
```
┌──────────────────────┐
│  Frontend (WebView)   │
│  Dashboard, Camera,   │
│  Chat FAB, Analytics  │
└──────────┬─────────────┘
           │ Tauri IPC
┌──────────┴─────────────┐
│  Rust Backend         │
│  ┌─────────────────┐ │
│  │ SQLite DB       │ │
│  │ LLM Core        │ │
│  │ Receipt Process │ │
│  │ Predictor      │ │
│  │ RAG Knowledge  │ │
│  └─────────────────┘ │
└──────────────────────┘
```

---

## 3. Data Model

### User
- Settings: currency, country
- Categories: pre-built defaults + custom
- Income sources
- Savings goals

### Project
- name, total_budget
- categories[] (name)

### Receipt
- image_path
- total, tax, discount
- items[] (name, qty, price)
- detected_category
- project_id (optional)
- recurring_flag

### Subscription
- name, amount, frequency, next_expected_date
- detected_from_receipt_id

### BudgetAlert
- category_id, threshold, triggered_at

### SavingsGoal
- name, target_amount, monthly_allocation, current_progress

---

## 4. Default Categories

Food, Transport, Utilities, Entertainment, Shopping, Health, Other

---

## 5. Screens

### 5.1 Onboarding Walkthrough
1. Welcome → Set currency and country
2. Review default categories (can add/edit)
3. Add income sources (optional, can skip)
4. Finish → Dashboard

### 5.2 Dashboard
- Monthly overview: total spent vs budget
- Budget alerts (50%/80%/100% warnings)
- Quick stats: top category, recent transactions
- FAB (floating action button) for quick add
- Disposable income remaining

### 5.3 Camera / Receipt Scanner
1. Open camera → capture
2. Preview: Retake or Confirm
3. AI processing → extraction results
4. Review/Edit: total, tax, discount, items, category
5. Assign to category or project
6. Confirm → Save

### 5.4 Projects View
- List of all projects
- Each shows: name, budget, spent, remaining
- Tap project → see category breakdown
- Add new project flow

### 5.5 Income & Subscriptions
- Income sources list (name, amount, frequency, next date)
- Detected subscriptions (from receipts)
- Savings goals with progress bars
- Monthly disposable calculation

### 5.6 Analytics
- Daily/monthly expense graphs
- Breakdown by category (pie/bar charts)
- Trends over time

### 5.7 Chat Panel (FAB Expand)
- Persistent chat window (doesn't reset on minimize unless user says)
- Powered by local LLM + RAG knowledge base
- Queries user's spending data
- Financial tips from embedded knowledge base

---

## 6. Receipt Flow

```
Camera opens
    ↓
Snap photo → Preview (Retake/Confirm)
    ↓
LLM extracts:
  - Total, tax, discount
  - Line items
  - Suggested category
  - Recurring detection
    ↓
Review screen: edit data, assign category/project
    ↓
Confirm → Save to SQLite + image to blob
    ↓
If recurring detected → Prompt: "Save as subscription?"
```

---

## 7. LLM Features

### Receipt Processing
- OCR-free extraction via vision-capable model or structured output from image
- Categorization based on line items and vendor
- Recurring detection: same vendor + similar amount + monthly interval

### Chat
- User spending insights
- Budget advice
- Category warnings
- Comparison to averages

### RAG Knowledge Base
- Pre-loaded financial tips
- Indexed for fast retrieval
- LLM retrieves relevant tips to augment responses

---

## 8. Features Detail

### F1: Income Flow Tracking
- User inputs income sources (amount, frequency)
- Monthly disposable = Total Income - Fixed Bills - Savings Goal
- Shows "Left to spend: $X"

### F2: Subscription Tracker
- LLM detects recurring from receipt patterns
- User confirms subscription → saved with frequency
- Alerts before due date

### F3: Budget Alerts
- Per category/project thresholds: 50%, 80%, 100%
- Visual indicators + push notifications
- Dashboard warning badges

### F4: Savings Goals
- Name, target amount, monthly allocation
- Progress bar visualization
- Tracking against target

---

## 9. Tech Notes

- **LLM binding:** llama.cpp via Rust FFI (llama-bindings or custom)
- **RAG:** Local vector embeddings stored in SQLite or separate storage
- **Images:** Stored in app's document directory, path in SQLite
- **Export:** JSON and CSV/XLSX for transaction data

---

## 10. Out of Scope (v1)

- Multi-device sync
- Cloud backup
- Bank API integration
- Investment tracking
