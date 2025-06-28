import { 
  isValidWebsite,
  getSafeWebsiteUrl,
  getSafeAvatarUrl,
  getSafeDisplayName,
  sanitizeUserContent 
} from '../utils/validation';

describe('ProfilePage Security Tests', () => {
  describe('Website URL Validation', () => {
    test('should accept valid HTTPS websites', () => {
      expect(isValidWebsite('https://example.com')).toBe(true);
      expect(isValidWebsite('https://johndoe.dev')).toBe(true);
      expect(isValidWebsite('https://github.com/user')).toBe(true);
    });

    test('should accept valid HTTP websites', () => {
      expect(isValidWebsite('http://example.com')).toBe(true);
      expect(isValidWebsite('http://legacy-site.org')).toBe(true);
    });

    test('should reject JavaScript injection URLs', () => {
      expect(isValidWebsite('javascript:alert("XSS")')).toBe(false);
      expect(isValidWebsite('javascript:void(0)')).toBe(false);
      expect(isValidWebsite('javascript:window.location="http://evil.com"')).toBe(false);
    });

    test('should reject data URLs', () => {
      expect(isValidWebsite('data:text/html,<script>alert("XSS")</script>')).toBe(false);
      expect(isValidWebsite('data:application/javascript,alert("XSS")')).toBe(false);
    });

    test('should reject file and other dangerous protocols', () => {
      expect(isValidWebsite('file:///etc/passwd')).toBe(false);
      expect(isValidWebsite('ftp://malicious.com')).toBe(false);
      expect(isValidWebsite('vbscript:msgbox("XSS")')).toBe(false);
      expect(isValidWebsite('tel:+1234567890')).toBe(false);
      expect(isValidWebsite('mailto:test@evil.com')).toBe(false);
    });

    test('should reject localhost and private IPs', () => {
      expect(isValidWebsite('http://localhost:3000')).toBe(false);
      expect(isValidWebsite('https://127.0.0.1')).toBe(false);
      expect(isValidWebsite('http://192.168.1.1')).toBe(false);
      expect(isValidWebsite('https://10.0.0.1')).toBe(false);
      expect(isValidWebsite('http://172.16.0.1')).toBe(false);
      expect(isValidWebsite('https://0.0.0.0')).toBe(false);
      expect(isValidWebsite('http://[::1]')).toBe(false);
    });

    test('should handle invalid input types', () => {
      expect(isValidWebsite(null)).toBe(false);
      expect(isValidWebsite(undefined)).toBe(false);
      expect(isValidWebsite(123)).toBe(false);
      expect(isValidWebsite({})).toBe(false);
      expect(isValidWebsite('')).toBe(false);
    });
  });

  describe('Safe Website URL Generation', () => {
    test('should return valid URLs unchanged', () => {
      const validUrl = 'https://johndoe.dev';
      expect(getSafeWebsiteUrl(validUrl)).toBe(validUrl);
    });

    test('should return empty string for invalid URLs', () => {
      expect(getSafeWebsiteUrl('javascript:alert("XSS")')).toBe('');
      expect(getSafeWebsiteUrl('http://localhost')).toBe('');
      expect(getSafeWebsiteUrl('ftp://malicious.com')).toBe('');
      expect(getSafeWebsiteUrl(null)).toBe('');
    });
  });

  describe('ProfilePage XSS Prevention Integration', () => {
    test('should handle malicious profile data safely', () => {
      const maliciousProfile = {
        avatar: 'javascript:alert("Avatar XSS")',
        displayName: '<script>alert("Name XSS")</script>Hacker',
        bio: '<img src=x onerror=alert("Bio XSS")>Evil bio',
        location: '<script>alert("Location XSS")</script>Evil City',
        website: 'javascript:alert("Website XSS")'
      };

      // Test all profile fields are sanitized
      expect(getSafeAvatarUrl(maliciousProfile.avatar)).toBe('/default-avatar.png');
      expect(getSafeDisplayName(maliciousProfile.displayName)).toBe('Hacker');
      expect(sanitizeUserContent(maliciousProfile.bio)).toBe('Evil bio');
      expect(getSafeWebsiteUrl(maliciousProfile.website)).toBe('');
    });

    test('should preserve safe profile content', () => {
      const safeProfile = {
        avatar: 'https://avatars.githubusercontent.com/u/12345',
        displayName: 'John Doe',
        bio: '<p>Software engineer passionate about <strong>open source</strong>.</p>',
        location: 'San Francisco, CA',
        website: 'https://johndoe.dev'
      };

      expect(getSafeAvatarUrl(safeProfile.avatar)).toBe(safeProfile.avatar);
      expect(getSafeDisplayName(safeProfile.displayName)).toBe(safeProfile.displayName);
      expect(sanitizeUserContent(safeProfile.bio)).toBe(safeProfile.bio);
      expect(getSafeWebsiteUrl(safeProfile.website)).toBe(safeProfile.website);
    });

    test('should handle mixed safe and unsafe content', () => {
      const mixedProfile = {
        avatar: 'https://secure.gravatar.com/avatar/hash', // Safe
        displayName: '<script>alert(1)</script>John Doe', // Unsafe name but safe content
        bio: '<p>Good content</p><script>alert(2)</script>', // Mixed content
        website: 'https://example.com' // Safe
      };

      expect(getSafeAvatarUrl(mixedProfile.avatar)).toBe(mixedProfile.avatar);
      expect(getSafeDisplayName(mixedProfile.displayName)).toBe('John Doe');
      expect(sanitizeUserContent(mixedProfile.bio)).toBe('<p>Good content</p>');
      expect(getSafeWebsiteUrl(mixedProfile.website)).toBe(mixedProfile.website);
    });

    test('should handle extremely long inputs', () => {
      const longProfile = {
        displayName: 'a'.repeat(200), // Too long
        bio: 'b'.repeat(1000), // Very long
        location: 'c'.repeat(300), // Too long
      };

      const safeName = getSafeDisplayName(longProfile.displayName);
      const safeBio = sanitizeUserContent(longProfile.bio);
      
      expect(safeName.length).toBeLessThanOrEqual(50);
      expect(safeBio).toBe(longProfile.bio); // DOMPurify doesn't limit length
    });

    test('should handle special characters safely', () => {
      const specialProfile = {
        displayName: 'User & Co.',
        bio: 'Company: Johnson & Johnson <br> Location: R&D',
        location: 'City & State',
      };

      // These should be handled safely without breaking
      expect(() => getSafeDisplayName(specialProfile.displayName)).not.toThrow();
      expect(() => sanitizeUserContent(specialProfile.bio)).not.toThrow();
    });
  });

  describe('Error Handling and Edge Cases', () => {
    test('should handle null and undefined gracefully', () => {
      const nullProfile = {
        avatar: null,
        displayName: undefined,
        bio: null,
        website: undefined
      };

      expect(getSafeAvatarUrl(nullProfile.avatar)).toBe('/default-avatar.png');
      expect(getSafeDisplayName(nullProfile.displayName)).toBe('Anonymous User');
      expect(sanitizeUserContent(nullProfile.bio)).toBe('');
      expect(getSafeWebsiteUrl(nullProfile.website)).toBe('');
    });

    test('should handle empty strings', () => {
      const emptyProfile = {
        avatar: '',
        displayName: '',
        bio: '',
        website: ''
      };

      expect(getSafeAvatarUrl(emptyProfile.avatar)).toBe('/default-avatar.png');
      expect(getSafeDisplayName(emptyProfile.displayName)).toBe('Anonymous User');
      expect(sanitizeUserContent(emptyProfile.bio)).toBe('');
      expect(getSafeWebsiteUrl(emptyProfile.website)).toBe('');
    });

    test('should handle malformed URLs', () => {
      const malformedUrls = [
        'not-a-url',
        'http://',
        'https://.',
        'ftp//missing-colon',
        'http://[invalid-ipv6',
        'https://domain..com'
      ];

      malformedUrls.forEach(url => {
        expect(getSafeAvatarUrl(url)).toBe('/default-avatar.png');
        expect(getSafeWebsiteUrl(url)).toBe('');
      });
    });
  });

  describe('Security Headers and CSP Compliance', () => {
    test('should not generate content that violates CSP', () => {
      const testInputs = [
        '<script>alert(1)</script>',
        '<img src=x onerror=alert(1)>',
        '<div onclick="alert(1)">Click</div>',
        '<iframe src="javascript:alert(1)"></iframe>'
      ];

      testInputs.forEach(input => {
        const sanitized = sanitizeUserContent(input);
        // Should not contain script tags or event handlers
        expect(sanitized).not.toMatch(/<script/i);
        expect(sanitized).not.toMatch(/onerror=/i);
        expect(sanitized).not.toMatch(/onclick=/i);
        expect(sanitized).not.toMatch(/<iframe/i);
      });
    });

    test('should generate safe HTML that passes CSP nonce requirements', () => {
      const safeHtml = sanitizeUserContent('<p><strong>Safe</strong> content with <em>emphasis</em></p>');
      
      // Should only contain allowed tags
      expect(safeHtml).toMatch(/^<p><strong>Safe<\/strong> content with <em>emphasis<\/em><\/p>$/);
      
      // Should not contain any attributes
      expect(safeHtml).not.toMatch(/\s\w+=/);
    });
  });
});