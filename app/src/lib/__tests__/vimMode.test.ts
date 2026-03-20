/**
 * Vim Mode Tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { VimMode } from '../vimMode';

describe('VimMode', () => {
  let vimMode: VimMode;
  let actions: Array<{ action: string; args?: any }>;

  beforeEach(() => {
    actions = [];
    vimMode = new VimMode((action, args) => {
      actions.push({ action, args });
    });
    vimMode.isEnabled = true;
  });

  const createKeyEvent = (key: string, options: Partial<KeyboardEvent> = {}): KeyboardEvent => {
    return {
      key,
      ctrlKey: false,
      metaKey: false,
      shiftKey: false,
      target: document.body,
      preventDefault: vi.fn(),
      stopPropagation: vi.fn(),
      ...options,
    } as unknown as KeyboardEvent;
  };

  describe('Normal Mode - Navigation', () => {
    it('should navigate with h/j/k/l keys', () => {
      vimMode.handleKey(createKeyEvent('h'));
      expect(actions).toEqual([{ action: 'photo.prev', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('l'));
      expect(actions).toEqual([{ action: 'photo.next', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('j'));
      expect(actions).toEqual([{ action: 'photo.next_row', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('k'));
      expect(actions).toEqual([{ action: 'photo.prev_row', args: undefined }]);
    });

    it('should handle gg (first photo)', () => {
      vimMode.handleKey(createKeyEvent('g'));
      expect(actions).toEqual([]);

      vimMode.handleKey(createKeyEvent('g'));
      expect(actions).toEqual([{ action: 'photo.first', args: undefined }]);
    });

    it('should handle G (last photo)', () => {
      vimMode.handleKey(createKeyEvent('G'));
      expect(actions).toEqual([{ action: 'photo.last', args: undefined }]);
    });
  });

  describe('Normal Mode - Rating', () => {
    it('should set rating with number keys', () => {
      for (let i = 0; i <= 5; i++) {
        actions = [];
        vimMode.handleKey(createKeyEvent(i.toString()));
        expect(actions).toEqual([{ action: 'rate', args: i }]);
      }
    });
  });

  describe('Normal Mode - Flags', () => {
    it('should handle pick flag (p/P)', () => {
      vimMode.handleKey(createKeyEvent('p'));
      expect(actions).toEqual([{ action: 'flag.pick', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('P'));
      expect(actions).toEqual([{ action: 'flag.pick', args: undefined }]);
    });

    it('should handle reject flag (x/X)', () => {
      vimMode.handleKey(createKeyEvent('x'));
      expect(actions).toEqual([{ action: 'flag.reject', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('X'));
      expect(actions).toEqual([{ action: 'flag.reject', args: undefined }]);
    });

    it('should handle unflagged (u/U)', () => {
      vimMode.handleKey(createKeyEvent('u'));
      expect(actions).toEqual([{ action: 'flag.none', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('U'));
      expect(actions).toEqual([{ action: 'flag.none', args: undefined }]);
    });
  });

  describe('Normal Mode - Edit Operations', () => {
    it('should handle yy (copy settings)', () => {
      vimMode.handleKey(createKeyEvent('y'));
      expect(actions).toEqual([]);

      vimMode.handleKey(createKeyEvent('y'));
      expect(actions).toEqual([{ action: 'develop.copy', args: undefined }]);
    });

    it('should handle Ctrl+p (paste)', () => {
      vimMode.handleKey(createKeyEvent('p', { ctrlKey: true }));
      expect(actions).toEqual([{ action: 'develop.paste', args: undefined }]);
    });

    it('should handle undo/redo', () => {
      vimMode.handleKey(createKeyEvent('u', { ctrlKey: true }));
      expect(actions).toEqual([{ action: 'edit.undo', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('r', { ctrlKey: true }));
      expect(actions).toEqual([{ action: 'edit.redo', args: undefined }]);
    });

    it('should handle repeat (.)', () => {
      vimMode.handleKey(createKeyEvent('.'));
      expect(actions).toEqual([{ action: 'edit.repeat', args: undefined }]);
    });
  });

  describe('Command Mode', () => {
    beforeEach(() => {
      vimMode.handleKey(createKeyEvent(':'));
      actions = [];
    });

    it('should enter command mode with :', () => {
      expect(vimMode.currentState).toBe('command');
    });

    it('should handle :export command', () => {
      'export'.split('').forEach(char => vimMode.handleKey(createKeyEvent(char)));
      vimMode.handleKey(createKeyEvent('Enter'));

      expect(actions).toEqual([{ action: 'export.open', args: undefined }]);
      expect(vimMode.currentState).toBe('normal');
    });

    it('should handle :rate command', () => {
      'rate 5'.split('').forEach(char => vimMode.handleKey(createKeyEvent(char)));
      vimMode.handleKey(createKeyEvent('Enter'));

      expect(actions).toEqual([{ action: 'rate', args: 5 }]);
    });

    it('should handle :flag pick command', () => {
      'flag pick'.split('').forEach(char => vimMode.handleKey(createKeyEvent(char)));
      vimMode.handleKey(createKeyEvent('Enter'));

      expect(actions).toEqual([{ action: 'flag.pick', args: undefined }]);
    });

    it('should handle :preset command', () => {
      'preset mypreset'.split('').forEach(char => vimMode.handleKey(createKeyEvent(char)));
      vimMode.handleKey(createKeyEvent('Enter'));

      expect(actions).toEqual([{ action: 'preset.apply_by_name', args: 'mypreset' }]);
    });

    it('should exit command mode with Escape', () => {
      vimMode.handleKey(createKeyEvent('Escape'));
      expect(vimMode.currentState).toBe('normal');
      expect(vimMode.commandBufferContent).toBe('');
    });

    it('should handle backspace', () => {
      'test'.split('').forEach(char => vimMode.handleKey(createKeyEvent(char)));
      expect(vimMode.commandBufferContent).toBe('test');

      vimMode.handleKey(createKeyEvent('Backspace'));
      expect(vimMode.commandBufferContent).toBe('tes');
    });
  });

  describe('Visual Mode', () => {
    beforeEach(() => {
      vimMode.handleKey(createKeyEvent('v'));
      actions = [];
    });

    it('should enter visual mode with v', () => {
      expect(vimMode.currentState).toBe('visual');
    });

    it('should extend selection with h/j/k/l', () => {
      vimMode.handleKey(createKeyEvent('h'));
      expect(actions).toEqual([{ action: 'select.extend_prev', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('l'));
      expect(actions).toEqual([{ action: 'select.extend_next', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('j'));
      expect(actions).toEqual([{ action: 'select.extend_down', args: undefined }]);

      actions = [];
      vimMode.handleKey(createKeyEvent('k'));
      expect(actions).toEqual([{ action: 'select.extend_up', args: undefined }]);
    });

    it('should rate selection', () => {
      vimMode.handleKey(createKeyEvent('5'));
      expect(actions).toEqual([{ action: 'rate.selection', args: 5 }]);
    });

    it('should paste to selection', () => {
      vimMode.handleKey(createKeyEvent('p'));
      expect(actions).toEqual([{ action: 'develop.paste.selection', args: undefined }]);
    });

    it('should exit visual mode with Escape', () => {
      vimMode.handleKey(createKeyEvent('Escape'));
      expect(vimMode.currentState).toBe('normal');
    });
  });

  describe('Input Focus Handling', () => {
    it('should ignore keys when input is focused', () => {
      const input = document.createElement('input');
      const event = createKeyEvent('h', { target: input });

      const consumed = vimMode.handleKey(event);
      expect(consumed).toBe(false);
      expect(actions).toEqual([]);
    });

    it('should ignore keys when textarea is focused', () => {
      const textarea = document.createElement('textarea');
      const event = createKeyEvent('h', { target: textarea });

      const consumed = vimMode.handleKey(event);
      expect(consumed).toBe(false);
      expect(actions).toEqual([]);
    });
  });
});
