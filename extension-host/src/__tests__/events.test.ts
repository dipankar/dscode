/**
 * Tests for Events API
 */

import { EventEmitter, Disposable } from '../api/events';

describe('EventEmitter', () => {
  let emitter: EventEmitter<string>;

  beforeEach(() => {
    emitter = new EventEmitter<string>();
  });

  afterEach(() => {
    emitter.dispose();
  });

  describe('event subscription', () => {
    it('should call listener when event is fired', () => {
      const listener = jest.fn();
      emitter.event(listener);

      emitter.fire('test-data');

      expect(listener).toHaveBeenCalledWith('test-data');
      expect(listener).toHaveBeenCalledTimes(1);
    });

    it('should call multiple listeners', () => {
      const listener1 = jest.fn();
      const listener2 = jest.fn();

      emitter.event(listener1);
      emitter.event(listener2);

      emitter.fire('data');

      expect(listener1).toHaveBeenCalledWith('data');
      expect(listener2).toHaveBeenCalledWith('data');
    });

    it('should fire multiple events to same listener', () => {
      const listener = jest.fn();
      emitter.event(listener);

      emitter.fire('first');
      emitter.fire('second');
      emitter.fire('third');

      expect(listener).toHaveBeenCalledTimes(3);
      expect(listener).toHaveBeenNthCalledWith(1, 'first');
      expect(listener).toHaveBeenNthCalledWith(2, 'second');
      expect(listener).toHaveBeenNthCalledWith(3, 'third');
    });
  });

  describe('disposable subscription', () => {
    it('should return disposable when subscribing', () => {
      const disposable = emitter.event(jest.fn());

      expect(disposable).toBeDefined();
      expect(typeof disposable.dispose).toBe('function');
    });

    it('should stop receiving events after dispose', () => {
      const listener = jest.fn();
      const disposable = emitter.event(listener);

      emitter.fire('before');
      expect(listener).toHaveBeenCalledTimes(1);

      disposable.dispose();

      emitter.fire('after');
      expect(listener).toHaveBeenCalledTimes(1); // Still 1, not 2
    });

    it('should not affect other listeners when one disposes', () => {
      const listener1 = jest.fn();
      const listener2 = jest.fn();

      const disposable1 = emitter.event(listener1);
      emitter.event(listener2);

      disposable1.dispose();

      emitter.fire('data');

      expect(listener1).not.toHaveBeenCalled();
      expect(listener2).toHaveBeenCalledWith('data');
    });
  });

  describe('emitter dispose', () => {
    it('should clear all listeners on dispose', () => {
      const listener1 = jest.fn();
      const listener2 = jest.fn();

      emitter.event(listener1);
      emitter.event(listener2);

      emitter.dispose();

      emitter.fire('data');

      expect(listener1).not.toHaveBeenCalled();
      expect(listener2).not.toHaveBeenCalled();
    });
  });

  describe('with complex types', () => {
    it('should work with object events', () => {
      const objectEmitter = new EventEmitter<{ id: number; name: string }>();
      const listener = jest.fn();

      objectEmitter.event(listener);
      objectEmitter.fire({ id: 1, name: 'test' });

      expect(listener).toHaveBeenCalledWith({ id: 1, name: 'test' });
      objectEmitter.dispose();
    });

    it('should work with array events', () => {
      const arrayEmitter = new EventEmitter<number[]>();
      const listener = jest.fn();

      arrayEmitter.event(listener);
      arrayEmitter.fire([1, 2, 3]);

      expect(listener).toHaveBeenCalledWith([1, 2, 3]);
      arrayEmitter.dispose();
    });

    it('should work with void events', () => {
      const voidEmitter = new EventEmitter<void>();
      const listener = jest.fn();

      voidEmitter.event(listener);
      voidEmitter.fire();

      expect(listener).toHaveBeenCalled();
      voidEmitter.dispose();
    });
  });
});

describe('Disposable', () => {
  describe('static from', () => {
    it('should create disposable from multiple disposables', () => {
      const dispose1 = jest.fn();
      const dispose2 = jest.fn();

      const combined = Disposable.from(
        { dispose: dispose1 },
        { dispose: dispose2 }
      );

      combined.dispose();

      expect(dispose1).toHaveBeenCalled();
      expect(dispose2).toHaveBeenCalled();
    });

    it('should handle empty array', () => {
      const combined = Disposable.from();

      expect(() => combined.dispose()).not.toThrow();
    });
  });
});
