import { 
  isValidUrl, 
  isValidDisplayName, 
  getSafeAvatarUrl, 
  getSafeDisplayName,
  escapeHtml,
  sanitizeUserContent 
} from '../utils/validation';

describe('XSS Prevention Security Tests', () => {
  describe('URL Validation', () => {
    test('should accept valid HTTPS URLs', () => {
      expect(isValidUrl('https://example.com/image.jpg')).toBe(true);
      expect(isValidUrl('https://avatars.githubusercontent.com/u/12345')).toBe(true);
    });

    test('should accept valid HTTP URLs', () => {
      expect(isValidUrl('http://example.com/image.jpg')).toBe(true);
    });

    test('should reject malicious JavaScript URLs', () => {
      expect(isValidUrl('javascript:alert("XSS")')).toBe(false);
      expect(isValidUrl('javascript:void(0)')).toBe(false);
    });

    test('should reject data URLs', () => {
      expect(isValidUrl('data:text/html,<script>alert("XSS")</script>')).toBe(false);
      expect(isValidUrl('data:image/svg+xml,<svg><script>alert("XSS")</script></svg>')).toBe(false);
    });

    test('should reject other malicious protocols', () => {
      expect(isValidUrl('file:///etc/passwd')).toBe(false);
      expect(isValidUrl('ftp://malicious.com')).toBe(false);
      expect(isValidUrl('vbscript:msgbox("XSS")')).toBe(false);
    });

    test('should handle invalid input types', () => {
      expect(isValidUrl(null)).toBe(false);
      expect(isValidUrl(undefined)).toBe(false);
      expect(isValidUrl(123)).toBe(false);
      expect(isValidUrl({})).toBe(false);
    });
  });

  describe('Display Name Validation', () => {
    test('should accept valid display names', () => {
      expect(isValidDisplayName('John Doe')).toBe(true);
      expect(isValidDisplayName('jane_smith')).toBe(true);
      expect(isValidDisplayName('user-123')).toBe(true);
      expect(isValidDisplayName('Test.User')).toBe(true);
    });

    test('should reject names with HTML tags', () => {
      expect(isValidDisplayName('<script>alert("XSS")</script>')).toBe(false);
      expect(isValidDisplayName('John<img src=x onerror=alert(1)>')).toBe(false);
      expect(isValidDisplayName('<div>John</div>')).toBe(false);
    });

    test('should reject names with special characters', () => {
      expect(isValidDisplayName('John@Doe')).toBe(false);
      expect(isValidDisplayName('John#Doe')).toBe(false);
      expect(isValidDisplayName('John&Doe')).toBe(false);
      expect(isValidDisplayName('John|Doe')).toBe(false);
    });

    test('should reject overly long names', () => {
      const longName = 'a'.repeat(51);
      expect(isValidDisplayName(longName)).toBe(false);
    });

    test('should handle invalid input types', () => {
      expect(isValidDisplayName(null)).toBe(false);
      expect(isValidDisplayName(undefined)).toBe(false);
      expect(isValidDisplayName(123)).toBe(false);
      expect(isValidDisplayName('')).toBe(false);
    });
  });

  describe('Safe Avatar URL Generation', () => {
    test('should return trusted domain URLs unchanged', () => {
      const githubUrl = 'https://avatars.githubusercontent.com/u/12345';
      expect(getSafeAvatarUrl(githubUrl)).toBe(githubUrl);
      
      const gravatarUrl = 'https://secure.gravatar.com/avatar/hash';
      expect(getSafeAvatarUrl(gravatarUrl)).toBe(gravatarUrl);
    });

    test('should return default for untrusted domains', () => {
      expect(getSafeAvatarUrl('https://malicious.com/avatar.jpg')).toBe('/default-avatar.png');
      expect(getSafeAvatarUrl('https://evil.org/script.js')).toBe('/default-avatar.png');
    });

    test('should return default for malicious URLs', () => {
      expect(getSafeAvatarUrl('javascript:alert("XSS")')).toBe('/default-avatar.png');
      expect(getSafeAvatarUrl('data:text/html,<script>alert("XSS")</script>')).toBe('/default-avatar.png');
    });

    test('should handle invalid inputs', () => {
      expect(getSafeAvatarUrl(null)).toBe('/default-avatar.png');
      expect(getSafeAvatarUrl(undefined)).toBe('/default-avatar.png');
      expect(getSafeAvatarUrl('')).toBe('/default-avatar.png');
    });
  });

  describe('Safe Display Name Generation', () => {
    test('should return valid names unchanged', () => {
      expect(getSafeDisplayName('John Doe')).toBe('John Doe');
      expect(getSafeDisplayName('jane_smith')).toBe('jane_smith');
    });

    test('should sanitize malicious names', () => {
      expect(getSafeDisplayName('<script>alert("XSS")</script>John')).toBe('John');
      expect(getSafeDisplayName('John<img src=x onerror=alert(1)>')).toBe('John');
    });

    test('should return default for invalid names', () => {
      expect(getSafeDisplayName('')).toBe('Anonymous User');
      expect(getSafeDisplayName(null)).toBe('Anonymous User');
      expect(getSafeDisplayName(undefined)).toBe('Anonymous User');
    });

    test('should truncate long names', () => {
      const longName = 'a'.repeat(100);
      const result = getSafeDisplayName(longName);
      expect(result.length).toBeLessThanOrEqual(50);
    });
  });

  describe('HTML Escaping', () => {
    test('should escape HTML entities', () => {
      expect(escapeHtml('<script>alert("XSS")</script>')).toBe('&lt;script&gt;alert("XSS")&lt;/script&gt;');
      expect(escapeHtml('John & Jane')).toBe('John &amp; Jane');
      expect(escapeHtml('"quotes"')).toBe('&quot;quotes&quot;');
    });

    test('should handle empty and invalid inputs', () => {
      expect(escapeHtml('')).toBe('');
      expect(escapeHtml(null)).toBe('');
      expect(escapeHtml(undefined)).toBe('');
    });
  });

  describe('User Content Sanitization', () => {
    test('should allow safe HTML tags', () => {
      expect(sanitizeUserContent('<p>Hello world</p>')).toBe('<p>Hello world</p>');
      expect(sanitizeUserContent('<strong>Bold text</strong>')).toBe('<strong>Bold text</strong>');
      expect(sanitizeUserContent('<em>Italic text</em>')).toBe('<em>Italic text</em>');
    });

    test('should remove dangerous HTML tags', () => {
      expect(sanitizeUserContent('<script>alert("XSS")</script>')).toBe('');
      expect(sanitizeUserContent('<img src=x onerror=alert(1)>')).toBe('');
      expect(sanitizeUserContent('<iframe src="javascript:alert(1)"></iframe>')).toBe('');
    });

    test('should remove all attributes', () => {
      expect(sanitizeUserContent('<p onclick="alert(1)">Text</p>')).toBe('<p>Text</p>');
      expect(sanitizeUserContent('<strong style="color:red">Text</strong>')).toBe('<strong>Text</strong>');
    });

    test('should handle mixed content', () => {
      const input = '<p>Safe text</p><script>alert("XSS")</script><strong>More safe text</strong>';
      const expected = '<p>Safe text</p><strong>More safe text</strong>';
      expect(sanitizeUserContent(input)).toBe(expected);
    });
  });
});

describe('Integration Tests', () => {
  test('should handle complete user object safely', () => {
    const maliciousUser = {
      avatar: 'javascript:alert("XSS")',
      displayName: '<script>alert("XSS")</script>Hacker'
    };

    const safeAvatar = getSafeAvatarUrl(maliciousUser.avatar);
    const safeName = getSafeDisplayName(maliciousUser.displayName);

    expect(safeAvatar).toBe('/default-avatar.png');
    expect(safeName).toBe('Hacker');
  });

  test('should handle edge cases gracefully', () => {
    const edgeCases = [null, undefined, '', 0, false, {}, []];
    
    edgeCases.forEach(input => {
      expect(() => getSafeAvatarUrl(input)).not.toThrow();
      expect(() => getSafeDisplayName(input)).not.toThrow();
      expect(() => sanitizeUserContent(input)).not.toThrow();
    });
  });
});