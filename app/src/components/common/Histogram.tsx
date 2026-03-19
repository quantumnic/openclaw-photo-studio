import { createSignal, createEffect, onMount } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

interface HistogramData {
  red: number[];
  green: number[];
  blue: number[];
  luma: number[];
}

interface HistogramProps {
  photoId?: string;
  height?: number;
}

/**
 * Histogram component for RGB + Luminance display
 *
 * Displays RGB channels as semi-transparent colored bars and luminance as white.
 * If no photoId is provided or histogram data is unavailable, shows a flat grey line.
 */
export function Histogram(props: HistogramProps) {
  const [histogramData, setHistogramData] = createSignal<HistogramData | null>(null);
  let canvasRef: HTMLCanvasElement | undefined;

  // Fetch histogram data when photoId changes
  createEffect(async () => {
    const id = props.photoId;
    if (!id) {
      setHistogramData(null);
      return;
    }

    try {
      const data = await invoke<HistogramData>('compute_histogram', { photoId: id });
      setHistogramData(data);
    } catch (err) {
      console.error('Failed to load histogram:', err);
      setHistogramData(null);
    }
  });

  // Draw histogram when data changes
  createEffect(() => {
    const canvas = canvasRef;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const data = histogramData();
    const width = canvas.width;
    const height = canvas.height;

    // Clear canvas
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, width, height);

    if (!data) {
      // No data: draw flat grey line
      ctx.strokeStyle = '#555';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, height / 2);
      ctx.lineTo(width, height / 2);
      ctx.stroke();
      return;
    }

    // Find max value for scaling
    const maxValue = Math.max(
      Math.max(...data.red),
      Math.max(...data.green),
      Math.max(...data.blue),
      Math.max(...data.luma)
    );

    if (maxValue === 0) {
      // Empty histogram: draw flat line at bottom
      ctx.strokeStyle = '#555';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, height - 1);
      ctx.lineTo(width, height - 1);
      ctx.stroke();
      return;
    }

    // Draw each channel
    const drawChannel = (channelData: number[], color: string, alpha: number) => {
      ctx.fillStyle = color;
      ctx.globalAlpha = alpha;

      for (let i = 0; i < 256; i++) {
        const x = (i / 256) * width;
        const barHeight = (channelData[i] / maxValue) * height;
        const y = height - barHeight;

        ctx.fillRect(x, y, Math.ceil(width / 256), barHeight);
      }

      ctx.globalAlpha = 1.0;
    };

    // Draw RGB channels with transparency
    drawChannel(data.red, '#ff0000', 0.4);
    drawChannel(data.green, '#00ff00', 0.4);
    drawChannel(data.blue, '#0000ff', 0.4);

    // Draw luminance as white with lower opacity
    drawChannel(data.luma, '#ffffff', 0.6);
  });

  const height = props.height ?? 80;

  return (
    <div class="histogram-container" style={{ width: '100%', height: `${height}px` }}>
      <canvas
        ref={canvasRef}
        width={256}
        height={height}
        style={{
          width: '100%',
          height: '100%',
          'image-rendering': 'crisp-edges',
        }}
      />
    </div>
  );
}
