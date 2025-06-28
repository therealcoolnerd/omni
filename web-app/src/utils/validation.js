import DOMPurify from 'dompurify';

/**
 * Validates if a URL is safe for use in img src attributes
 * @param {string} url - The URL to validate
 * @returns {boolean} - Whether the URL is valid and safe
 */
export const isValidUrl = (url) => {
  if (!url || typeof url !== 'string') return false;
  
  try {
    const parsed = new URL(url);
    // Only allow HTTP/HTTPS protocols
    return ['http:', 'https:'].includes(parsed.protocol);
  } catch {
    return false;
  }
};

/**
 * Validates if a display name contains only safe characters
 * @param {string} name - The display name to validate
 * @returns {boolean} - Whether the name is valid
 */
export const isValidDisplayName = (name) => {
  if (!name || typeof name !== 'string') return false;
  
  // Only allow alphanumeric, spaces, and safe punctuation
  const safePattern = /^[a-zA-Z0-9\s\-_.]{1,50}$/;
  return safePattern.test(name);
};

/**
 * Gets a safe avatar URL with fallback to default
 * @param {string} avatarUrl - The user's avatar URL
 * @returns {string} - Safe avatar URL or default
 */
export const getSafeAvatarUrl = (avatarUrl) => {
  // Validate URL format and protocol
  if (!avatarUrl || !isValidUrl(avatarUrl)) {
    return '/default-avatar.png'; // Fallback to default
  }
  
  // Only allow specific trusted domains for avatars
  const allowedDomains = [
    'https://avatars.githubusercontent.com',
    'https://secure.gravatar.com',
    'https://cdn.discordapp.com',
    'https://images.unsplash.com'
  ];
  
  const isAllowedDomain = allowedDomains.some(domain => 
    avatarUrl.startsWith(domain)
  );
  
  return isAllowedDomain ? avatarUrl : '/default-avatar.png';
};

/**
 * Sanitizes display name to prevent XSS
 * @param {string} displayName - The user's display name
 * @returns {string} - Sanitized display name
 */
export const getSafeDisplayName = (displayName) => {
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

/**
 * Escapes HTML entities in text
 * @param {string} text - Text to escape
 * @returns {string} - HTML-escaped text
 */
export const escapeHtml = (text) => {
  if (!text || typeof text !== 'string') return '';
  
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
};

/**
 * Sanitizes user-generated content for safe display
 * @param {string} content - User content to sanitize
 * @returns {string} - Sanitized content
 */
export const sanitizeUserContent = (content) => {
  if (!content || typeof content !== 'string') return '';
  
  return DOMPurify.sanitize(content, {
    ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'u'],
    ALLOWED_ATTR: []
  });
};

/**
 * Validates if a website URL is safe
 * @param {string} url - The website URL to validate
 * @returns {boolean} - Whether the URL is valid and safe
 */
export const isValidWebsite = (url) => {
  if (!url || typeof url !== 'string') return false;
  
  try {
    const parsed = new URL(url);
    // Only allow HTTP/HTTPS protocols
    if (!['http:', 'https:'].includes(parsed.protocol)) return false;
    
    // Block suspicious domains
    const suspiciousDomains = [
      'localhost',
      '127.0.0.1',
      '0.0.0.0',
      '::1'
    ];
    
    const hostname = parsed.hostname.toLowerCase();
    if (suspiciousDomains.includes(hostname)) return false;
    
    // Block private IP ranges
    if (hostname.match(/^(10\.|172\.(1[6-9]|2[0-9]|3[01])\.|192\.168\.)/)) return false;
    
    return true;
  } catch {
    return false;
  }
};

/**
 * Gets a safe website URL or returns empty string
 * @param {string} websiteUrl - The website URL to validate
 * @returns {string} - Safe website URL or empty string
 */
export const getSafeWebsiteUrl = (websiteUrl) => {
  return isValidWebsite(websiteUrl) ? websiteUrl : '';
};