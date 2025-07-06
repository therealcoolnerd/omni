import DOMPurify from 'dompurify';
import { getSafeDisplayName, sanitizeUserContent } from '../utils/validation';

describe('DOMPurify Integration', () => {
  test('getSafeDisplayName output matches DOMPurify sanitization', () => {
    const input = '<script>alert(1)</script>John';
    const expected = DOMPurify.sanitize(input, {
      ALLOWED_TAGS: [],
      ALLOWED_ATTR: []
    }).trim().substring(0, 50) || 'Anonymous User';
    expect(getSafeDisplayName(input)).toBe(expected);
  });

  test('sanitizeUserContent output matches DOMPurify sanitization', () => {
    const content = '<p onclick="alert(1)">hi</p>';
    const expected = DOMPurify.sanitize(content, {
      ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'u'],
      ALLOWED_ATTR: []
    });
    expect(sanitizeUserContent(content)).toBe(expected);
  });
});
