// Suppress SES lockdown warnings
const originalWarn = console.warn;
console.warn = (...args) => {
  if (args[0]?.includes?.('SES') || args[0]?.includes?.('Removing intrinsics')) return;
  originalWarn.apply(console, args);
};

// Suppress CSP warnings in development
const originalError = console.error;
console.error = (...args) => {
  if (args[0]?.includes?.('Content-Security-Policy')) return;
  originalError.apply(console, args);
};

import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App.tsx';
import './index.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);