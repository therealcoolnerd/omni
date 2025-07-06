import fs from 'fs';
import path from 'path';

describe('CSP Meta Tag', () => {
  test('public index.html includes Content Security Policy header', () => {
    const filePath = path.resolve(__dirname, '../../public/index.html');
    const html = fs.readFileSync(filePath, 'utf8');
    expect(html).toMatch(/<meta[^>]+http-equiv=["']Content-Security-Policy["']/i);
  });
});
