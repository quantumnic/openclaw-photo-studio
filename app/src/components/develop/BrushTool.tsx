/**
 * Adjustment Brush Tool Component
 *
 * Interactive brush for painting local adjustments on images.
 * Features:
 * - Variable brush size, feather, and flow
 * - Erase mode (Alt key)
 * - Real-time cursor preview
 * - Mask overlay visualization
 */

import { createSignal, Show, onMount, onCleanup } from 'solid-js';

interface BrushToolProps {
  imageWidth: number;
  imageHeight: number;
  onBrushStroke?: (stroke: BrushStroke) => void;
}

export interface BrushStroke {
  points: Array<{ x: number; y: number }>;
  size: number;
  feather: number;
  flow: number;
  isErasing: boolean;
}

export function BrushTool(props: BrushToolProps) {
  const [brushActive, setBrushActive] = createSignal(false);
  const [brushSize, setBrushSize] = createSignal(50); // 0-100, relative to image
  const [brushFeather, setBrushFeather] = createSignal(50); // 0-100
  const [brushFlow, setBrushFlow] = createSignal(100); // 0-100
  const [isErasing, setIsErasing] = createSignal(false);
  const [currentStroke, setCurrentStroke] = createSignal<Array<{ x: number; y: number }>>([]);
  const [brushCursor, setBrushCursor] = createSignal<{ x: number; y: number } | null>(null);
  const [isPainting, setIsPainting] = createSignal(false);

  let canvasRef: HTMLCanvasElement | undefined;

  const handleMouseMove = (e: MouseEvent) => {
    if (!brushActive()) return;

    const rect = (e.target as HTMLElement).getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    const y = (e.clientY - rect.top) / rect.height;

    setBrushCursor({ x, y });

    if (isPainting() && e.buttons === 1) {
      setCurrentStroke((s) => [...s, { x, y }]);
    }
  };

  const handleMouseDown = (e: MouseEvent) => {
    if (!brushActive() || e.button !== 0) return;

    setIsPainting(true);
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    const y = (e.clientY - rect.top) / rect.height;

    setCurrentStroke([{ x, y }]);
  };

  const handleMouseUp = () => {
    if (!isPainting()) return;

    setIsPainting(false);

    const stroke = currentStroke();
    if (stroke.length > 0 && brushActive()) {
      // Call the callback with the completed stroke
      props.onBrushStroke?.({
        points: stroke,
        size: brushSize(),
        feather: brushFeather(),
        flow: brushFlow(),
        isErasing: isErasing(),
      });

      setCurrentStroke([]);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Alt') {
      setIsErasing(true);
    }
  };

  const handleKeyUp = (e: KeyboardEvent) => {
    if (e.key === 'Alt') {
      setIsErasing(false);
    }
  };

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
  });

  onCleanup(() => {
    window.removeEventListener('keydown', handleKeyDown);
    window.removeEventListener('keyup', handleKeyUp);
  });

  return (
    <div class="brush-tool">
      {/* Brush Settings Panel */}
      <Show when={brushActive()}>
        <div class="brush-settings absolute top-4 right-4 bg-black/80 rounded-lg p-4 text-white z-10">
          <h3 class="text-sm font-semibold mb-3">Adjustment Brush</h3>

          <div class="space-y-3">
            <div>
              <label class="text-xs block mb-1">Size: {brushSize()}</label>
              <input
                type="range"
                min="1"
                max="100"
                value={brushSize()}
                onInput={(e) => setBrushSize(parseInt(e.currentTarget.value))}
                class="w-full"
              />
            </div>

            <div>
              <label class="text-xs block mb-1">Feather: {brushFeather()}</label>
              <input
                type="range"
                min="0"
                max="100"
                value={brushFeather()}
                onInput={(e) => setBrushFeather(parseInt(e.currentTarget.value))}
                class="w-full"
              />
            </div>

            <div>
              <label class="text-xs block mb-1">Flow: {brushFlow()}</label>
              <input
                type="range"
                min="1"
                max="100"
                value={brushFlow()}
                onInput={(e) => setBrushFlow(parseInt(e.currentTarget.value))}
                class="w-full"
              />
            </div>

            <div class="flex items-center gap-2 pt-2 border-t border-white/20">
              <input
                type="checkbox"
                id="erase-toggle"
                checked={isErasing()}
                onChange={(e) => setIsErasing(e.currentTarget.checked)}
              />
              <label for="erase-toggle" class="text-xs">
                Erase Mode (Alt)
              </label>
            </div>

            <button
              class="w-full mt-2 px-3 py-1.5 text-xs bg-blue-600 hover:bg-blue-700 rounded"
              onClick={() => {
                // Reset/New Brush
                setCurrentStroke([]);
              }}
            >
              Reset Brush
            </button>
          </div>
        </div>
      </Show>

      {/* Brush Cursor Overlay (SVG) */}
      <Show when={brushActive() && brushCursor()}>
        {(cursor) => (
          <svg
            class="absolute inset-0 pointer-events-none z-20"
            style={{
              width: '100%',
              height: '100%',
            }}
          >
            <BrushCursor
              x={cursor().x * props.imageWidth}
              y={cursor().y * props.imageHeight}
              size={brushSize()}
              feather={brushFeather()}
              isErasing={isErasing()}
            />
          </svg>
        )}
      </Show>

      {/* Canvas Overlay for Painting */}
      <canvas
        ref={canvasRef}
        class="absolute inset-0 z-10"
        style={{
          cursor: brushActive() ? 'none' : 'default',
          width: '100%',
          height: '100%',
        }}
        onMouseMove={handleMouseMove}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      />
    </div>
  );
}

/**
 * Brush Cursor Component
 * Shows two concentric circles: inner (hard edge) and outer (feather edge)
 */
function BrushCursor(props: {
  x: number;
  y: number;
  size: number;
  feather: number;
  isErasing: boolean;
}) {
  // Calculate actual pixel size (size is relative 0-100)
  const pixelSize = (props.size / 100) * 200; // Max 200px brush
  const outerRadius = pixelSize / 2;
  const innerRadius = outerRadius * (1 - props.feather / 100);

  const color = props.isErasing ? '#ff4444' : '#ffffff';

  return (
    <g>
      {/* Outer circle (feather edge) */}
      <circle
        cx={props.x}
        cy={props.y}
        r={outerRadius}
        fill="none"
        stroke={color}
        stroke-width="1"
        stroke-opacity="0.5"
        style={{ filter: 'drop-shadow(0 0 2px rgba(0,0,0,0.8))' }}
      />

      {/* Inner circle (hard edge) */}
      {props.feather > 0 && (
        <circle
          cx={props.x}
          cy={props.y}
          r={innerRadius}
          fill="none"
          stroke={color}
          stroke-width="1"
          stroke-opacity="0.8"
          style={{ filter: 'drop-shadow(0 0 2px rgba(0,0,0,0.8))' }}
        />
      )}

      {/* Center dot */}
      <circle
        cx={props.x}
        cy={props.y}
        r="2"
        fill={color}
        opacity="0.8"
        style={{ filter: 'drop-shadow(0 0 2px rgba(0,0,0,0.8))' }}
      />
    </g>
  );
}

export default BrushTool;
