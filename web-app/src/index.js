import React from 'react';
import ReactDOM from 'react-dom/client';
import './App.css';
import './theme.css';
import App from './App';

// Security: Strict mode helps catch potential issues
const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);