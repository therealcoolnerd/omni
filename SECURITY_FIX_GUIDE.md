# XSS Vulnerability Fix Guide

## Issue: DOM text reinterpreted as HTML (js/xss-through-dom)

### Problem
The CodeQL alert indicates that user-controlled data is being used in a context where it could be interpreted as HTML, leading to potential Cross-Site Scripting (XSS) vulnerabilities.

### Location
- File: `web-app/src/pages/TimelinePage.jsx:162`
- Issue: `user.avatar` and `user.displayName` are used without proper sanitization

### Secure Fix Implementation

#### 1. Input Sanitization and Validation

```jsx
import DOMPurify from 'dompurify';
import { isValidUrl, isValidDisplayName } from '../utils/validation';

// Safe avatar URL validation
const getSafeAvatarUrl = (avatarUrl) => {
  // Validate URL format and protocol
  if (!avatarUrl || !isValidUrl(avatarUrl)) {
    return '/default-avatar.png'; // Fallback to default
  }
  
  // Only allow specific trusted domains for avatars
  const allowedDomains = [
    'https://your-trusted-cdn.com',
    'https://secure.gravatar.com',
    'https://avatars.githubusercontent.com'
  ];
  
  const url = new URL(avatarUrl);
  const isAllowedDomain = allowedDomains.some(domain => 
    avatarUrl.startsWith(domain)
  );
  
  return isAllowedDomain ? avatarUrl : '/default-avatar.png';
};

// Safe display name sanitization
const getSafeDisplayName = (displayName) => {
  if (!displayName || !isValidDisplayName(displayName)) {
    return 'Anonymous User';
  }
  
  // Sanitize HTML and limit length
  const sanitized = DOMPurify.sanitize(displayName, { 
    ALLOWED_TAGS: [],  // No HTML tags allowed
    ALLOWED_ATTR: []   // No attributes allowed
  });
  
  return sanitized.substring(0, 50); // Limit length
};
```

#### 2. Secure Component Implementation

```jsx
// SECURE VERSION
const TimelinePost = ({ user, post }) => {
  const safeAvatar = getSafeAvatarUrl(user.avatar);
  const safeDisplayName = getSafeDisplayName(user.displayName);
  
  return (
    <form onSubmit={handleCreatePost} className="space-y-4">
      <div className="flex items-start space-x-3">
        <img
          src={safeAvatar}
          alt={safeDisplayName}
          className="w-12 h-12 rounded-full border-2 border-nukie-gold/30"
          onError={(e) => {
            // Fallback if image fails to load
            e.target.src = '/default-avatar.png';
          }}
        />
        <div className="flex-1">
          <span className="font-medium text-white">
            {safeDisplayName}
          </span>
          {/* Rest of component */}
        </div>
      </div>
    </form>
  );
};
```

#### 3. Validation Utilities

```jsx
// utils/validation.js
export const isValidUrl = (url) => {
  try {
    const parsed = new URL(url);
    return ['http:', 'https:'].includes(parsed.protocol);
  } catch {
    return false;
  }
};

export const isValidDisplayName = (name) => {
  if (!name || typeof name !== 'string') return false;
  
  // Only allow alphanumeric, spaces, and safe punctuation
  const safePattern = /^[a-zA-Z0-9\s\-_.]{1,50}$/;
  return safePattern.test(name);
};

// Escape HTML entities
export const escapeHtml = (text) => {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
};
```

#### 4. Content Security Policy (CSP)

Add to your HTML head or server headers:

```html
<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self'; 
               img-src 'self' https://trusted-cdn.com https://secure.gravatar.com; 
               script-src 'self'; 
               style-src 'self' 'unsafe-inline';">
```

#### 5. React-Specific Security Measures

```jsx
// Use React's built-in escaping
const SafeComponent = ({ userContent }) => {
  return (
    <div>
      {/* React automatically escapes this */}
      <span>{userContent}</span>
      
      {/* NEVER use dangerouslySetInnerHTML with user content */}
      {/* <div dangerouslySetInnerHTML={{__html: userContent}} /> */}
    </div>
  );
};
```

### Dependencies to Add

```bash
npm install dompurify
npm install --save-dev @types/dompurify  # For TypeScript
```

### Testing the Fix

```jsx
// test/security.test.js
import { getSafeAvatarUrl, getSafeDisplayName } from '../utils/validation';

describe('XSS Prevention', () => {
  test('should sanitize malicious avatar URLs', () => {
    const maliciousUrl = 'javascript:alert("XSS")';
    expect(getSafeAvatarUrl(maliciousUrl)).toBe('/default-avatar.png');
  });
  
  test('should sanitize malicious display names', () => {
    const maliciousName = '<script>alert("XSS")</script>John';
    expect(getSafeDisplayName(maliciousName)).toBe('John');
  });
  
  test('should handle data URLs safely', () => {
    const dataUrl = 'data:text/html,<script>alert("XSS")</script>';
    expect(getSafeAvatarUrl(dataUrl)).toBe('/default-avatar.png');
  });
});
```

## Implementation Steps

1. **Install dependencies**: `npm install dompurify`
2. **Create validation utilities**: Add the validation functions
3. **Update the component**: Replace direct usage with sanitized versions
4. **Add CSP headers**: Implement Content Security Policy
5. **Test thoroughly**: Add security tests and manual testing
6. **Code review**: Have security-focused code review

## Prevention Checklist

- [ ] All user inputs are validated and sanitized
- [ ] No direct HTML injection from user data
- [ ] URL validation for all external resources
- [ ] Content Security Policy implemented
- [ ] Error handling for invalid inputs
- [ ] Security tests added
- [ ] Regular dependency updates for security patches

This fix will resolve the CodeQL alert and prevent XSS vulnerabilities in your React application.