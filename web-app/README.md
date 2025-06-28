# Omni Web App - Secure React Application

This is the secure web application for the Omni package manager, implementing comprehensive XSS protection and security best practices.

## 🔒 Security Features

### XSS Prevention
- **Input Sanitization**: All user inputs are validated and sanitized using DOMPurify
- **URL Validation**: Avatar URLs are validated against trusted domains only
- **Content Security Policy**: Strict CSP headers prevent script injection
- **HTML Escaping**: All user content is properly escaped before rendering

### Security Headers
- `Content-Security-Policy`: Restricts resource loading to trusted sources
- `X-Content-Type-Options: nosniff`: Prevents MIME type sniffing
- `X-Frame-Options: DENY`: Prevents clickjacking attacks
- `X-XSS-Protection`: Enables browser XSS filtering

### Input Validation
- Display names: Alphanumeric characters, spaces, hyphens, underscores, periods only
- URLs: HTTPS/HTTP protocols only, trusted domains for avatars
- Content: HTML sanitization with allowed tags only

## 🚀 Getting Started

### Prerequisites
- Node.js 16+ 
- npm or yarn

### Installation
```bash
cd web-app
npm install
```

### Development
```bash
npm start
```

### Testing
```bash
npm test
```

### Security Testing
Run the comprehensive security test suite:
```bash
npm test src/tests/security.test.js
```

## 📁 Project Structure

```
web-app/
├── src/
│   ├── pages/
│   │   ├── TimelinePage.jsx     # Main timeline with secure implementation
│   │   └── ProfilePage.jsx      # User profile with XSS protection
│   ├── utils/
│   │   └── validation.js        # Security validation utilities
│   ├── tests/
│   │   ├── security.test.js     # Timeline security tests
│   │   └── profilePage.security.test.js  # Profile page security tests
│   ├── App.jsx                  # Main app component with routing
│   └── index.js                 # Entry point
├── public/
│   └── index.html              # HTML with security headers
└── package.json                # Dependencies with security packages
```

## 🛡️ Security Implementation Details

### 1. Avatar URL Security
- Only allows URLs from trusted domains (GitHub, Gravatar, etc.)
- Rejects javascript:, data:, and other dangerous protocols
- Falls back to default avatar for invalid URLs

### 2. Display Name Sanitization
- Strips all HTML tags and dangerous characters
- Limits length to 50 characters
- Falls back to "Anonymous User" for invalid names

### 3. Content Sanitization
- Uses DOMPurify to clean user-generated content
- Only allows safe HTML tags: `<p>`, `<br>`, `<strong>`, `<em>`, `<u>`
- Removes all attributes to prevent event handlers

### 4. Error Handling
- All validation functions handle null/undefined gracefully
- Image loading errors fall back to default avatar
- Network errors are caught and logged

## 🧪 Testing

The application includes comprehensive security tests covering:
- URL validation (malicious protocols, untrusted domains)
- Display name validation (HTML injection, special characters)
- Content sanitization (script tags, dangerous HTML)
- Integration tests for complete user objects
- Edge case handling

## 📋 CodeQL Alert Resolution

This implementation specifically addresses multiple CodeQL alerts:

### Alert 1: TimelinePage.jsx:162
- **Rule ID**: `js/xss-through-dom`
- **Issue**: DOM text reinterpreted as HTML without escaping
- **Fix**: Comprehensive input validation and sanitization
- **Line 162**: Now uses `getSafeAvatarUrl()` and `getSafeDisplayName()`

### Alert 2: ProfilePage.jsx:37  
- **Rule ID**: `js/xss-through-dom`
- **Issue**: Avatar and display name used without escaping
- **Fix**: Same validation utilities applied to profile page
- **Line 37**: Now uses secure avatar and display name rendering

### Additional Security Measures
- Website URL validation with private IP blocking
- Bio content sanitization without dangerouslySetInnerHTML
- Form input validation and length limiting
- Comprehensive error handling and fallbacks

## 🔄 Deployment Security

For production deployment, ensure:
1. CSP headers are enforced at the server level
2. HTTPS is enabled for all connections
3. Security headers are properly configured
4. Regular dependency updates for security patches
5. Monitor for new XSS vulnerabilities

## 📝 Security Checklist

- [x] Input validation for all user data
- [x] URL validation for external resources
- [x] HTML sanitization using DOMPurify
- [x] Content Security Policy implementation
- [x] Security headers configuration
- [x] Comprehensive security testing
- [x] Error handling for security failures
- [x] Default fallbacks for invalid data
- [x] Regular dependency security updates
- [x] No use of `dangerouslySetInnerHTML`