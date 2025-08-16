import { describe, it, expect, beforeEach } from 'vitest';
import { 
  slugify, 
  AttrPromise, 
  getCurrentLanguage, 
  generateUuidv5, 
  setCurrentLanguage 
} from '../js/utils';

describe('Utils', () => {
  beforeEach(() => {
    // Reset language to default
    setCurrentLanguage('en');
  });

  describe('slugify', () => {
    it('should convert strings to URL-friendly slugs', () => {
      expect(slugify('Hello World!')).toBe('HelloWorld');
      expect(slugify('Test-String_123')).toBe('TestString_123');
      expect(slugify('Special@#$%Characters')).toBe('SpecialCharacters');
    });

    it('should handle numbers', () => {
      expect(slugify(123)).toBe('123');
      expect(slugify(12.34)).toBe('1234');
    });

    it('should limit length to SLUG_LENGTH', () => {
      const longString = 'a'.repeat(50);
      const slug = slugify(longString);
      expect(slug.length).toBeLessThanOrEqual(20);
    });

    it('should handle empty strings', () => {
      expect(slugify('')).toBe('');
      expect(slugify(null)).toBe('null');
      expect(slugify(undefined)).toBe('undefined');
    });
  });

  describe('getCurrentLanguage', () => {
    it('should return current language when set', () => {
      setCurrentLanguage('fr');
      expect(getCurrentLanguage()).toBe('fr');
    });

    it('should return default language when not set', () => {
      setCurrentLanguage(''); // Reset
      expect(getCurrentLanguage()).toBe('en');
    });

    it('should extract language code from locale', () => {
      // Mock navigator.language
      const originalNavigator = global.navigator;
      global.navigator = { language: 'en-US' } as any;
      
      setCurrentLanguage(''); // Reset
      expect(getCurrentLanguage()).toBe('en');
      
      global.navigator = originalNavigator;
    });
  });

  describe('setCurrentLanguage', () => {
    it('should set the current language', () => {
      setCurrentLanguage('es');
      expect(getCurrentLanguage()).toBe('es');
      
      setCurrentLanguage('de');
      expect(getCurrentLanguage()).toBe('de');
    });
  });

  describe('generateUuidv5', () => {
    it('should generate UUID v5 for simple keys', () => {
      const result = generateUuidv5(['resource', 'test-id'], 'simple-key');
      expect(typeof result).toBe('string');
      expect(result).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
    });

    it('should generate consistent UUIDs for same input', () => {
      const result1 = generateUuidv5(['resource', 'test-id'], 'consistent-key');
      const result2 = generateUuidv5(['resource', 'test-id'], 'consistent-key');
      expect(result1).toBe(result2);
    });

    it('should generate different UUIDs for different inputs', () => {
      const result1 = generateUuidv5(['resource', 'test-id'], 'key1');
      const result2 = generateUuidv5(['resource', 'test-id'], 'key2');
      expect(result1).not.toBe(result2);
    });

    it('should handle array keys', () => {
      const result = generateUuidv5(['graph', 'graph-id'], ['key1', 'key2', 'key3']);
      expect(typeof result).toBe('string');
      expect(result).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
    });

    it('should handle long key arrays with compression', () => {
      const longKeys = Array(100).fill(null).map((_, i) => `very-long-key-name-${i}`);
      const result = generateUuidv5(['test', 'compression'], longKeys);
      expect(typeof result).toBe('string');
      expect(result).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
    });
  });

  describe('AttrPromise', () => {
    it('should be a Promise', () => {
      const promise = new AttrPromise((resolve) => resolve('test'));
      expect(promise).toBeInstanceOf(Promise);
    });

    it('should resolve like a normal Promise', async () => {
      const promise = new AttrPromise((resolve) => resolve('test-value'));
      const result = await promise;
      expect(result).toBe('test-value');
    });

    it('should allow property access through proxy', async () => {
      const testObject = { name: 'Test', value: 42 };
      const promise = new AttrPromise((resolve) => resolve(testObject));
      
      // Property access should return a Promise
      const namePromise = promise.name;
      const valuePromise = promise.value;
      
      expect(namePromise).toBeInstanceOf(Promise);
      expect(valuePromise).toBeInstanceOf(Promise);
      
      expect(await namePromise).toBe('Test');
      expect(await valuePromise).toBe(42);
    });

    it('should allow property setting through proxy', async () => {
      const testObject = { name: 'Original' };
      const promise = new AttrPromise((resolve) => resolve(testObject));
      
      // Set property
      promise.name = 'Modified';
      
      // Wait for the promise to resolve and check the modified value
      const resolved = await promise;
      await new Promise(resolve => setTimeout(resolve, 0)); // Allow async set to complete
      
      expect((resolved as any).name).toBe('Modified');
    });

    it('should handle method calls through proxy', async () => {
      const testObject = {
        value: 10,
        getValue() { return this.value; },
        increment() { this.value++; return this.value; }
      };
      
      const promise = new AttrPromise((resolve) => resolve(testObject));
      
      // Wait for the promise to resolve first, then access methods
      const resolved = await promise;
      expect((resolved as any).getValue()).toBe(10);
      expect((resolved as any).increment()).toBe(11);
    });

    it('should handle non-existent properties gracefully', async () => {
      const testObject = { existing: 'value' };
      const promise = new AttrPromise((resolve) => resolve(testObject));
      
      const nonExistentPromise = promise.nonExistent;
      expect(await nonExistentPromise).toBeUndefined();
    });

    it('should handle null/undefined resolved values', async () => {
      const nullPromise = new AttrPromise((resolve) => resolve(null));
      const undefinedPromise = new AttrPromise((resolve) => resolve(undefined));
      
      const nullPropertyPromise = nullPromise.someProperty;
      const undefinedPropertyPromise = undefinedPromise.someProperty;
      
      expect(await nullPropertyPromise).toBeNull();
      expect(await undefinedPropertyPromise).toBeUndefined();
    });

    it('should reject like a normal Promise', async () => {
      const promise = new AttrPromise((_, reject) => reject(new Error('Test error')));
      
      await expect(promise).rejects.toThrow('Test error');
    });

    it('should maintain promise chain functionality', async () => {
      const promise = new AttrPromise((resolve) => resolve(5));
      
      const doubled = promise.then((val: any) => val * 2);
      const stringified = doubled.then((val: any) => val.toString());
      
      expect(await stringified).toBe('10');
    });
  });
});
