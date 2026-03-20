/**
 * Secondary Display View
 *
 * Minimal view for second monitor: just the image, no panels.
 * Shows the currently selected photo in full screen.
 */

import { createSignal, Show, onMount } from 'solid-js';

export function SecondaryView() {
  const [currentPhotoId, setCurrentPhotoId] = createSignal<string | null>(null);
  const [imageUrl, setImageUrl] = createSignal<string>('');

  onMount(() => {
    // Get photo ID from URL params
    const params = new URLSearchParams(window.location.search);
    const photoId = params.get('photo');

    if (photoId) {
      setCurrentPhotoId(photoId);
      // TODO: Load image from backend
      // For now, placeholder
      setImageUrl(`/api/photos/${photoId}/preview`);
    }

    // Listen for updates from main window
    window.addEventListener('message', (event) => {
      if (event.data.type === 'UPDATE_PHOTO') {
        setCurrentPhotoId(event.data.photoId);
        setImageUrl(`/api/photos/${event.data.photoId}/preview`);
      }
    });
  });

  return (
    <div class="w-screen h-screen bg-black flex items-center justify-center">
      <Show
        when={imageUrl()}
        fallback={
          <div class="text-white text-xl">
            No photo selected
          </div>
        }
      >
        <img
          src={imageUrl()}
          alt="Photo"
          class="max-w-full max-h-full object-contain"
        />
      </Show>

      {/* Minimal info overlay */}
      <div class="absolute top-4 right-4 text-white text-sm bg-black/50 px-3 py-2 rounded">
        Photo ID: {currentPhotoId() || 'None'}
      </div>
    </div>
  );
}

export default SecondaryView;
