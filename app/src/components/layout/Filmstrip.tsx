export function Filmstrip() {
  const photos = Array.from({ length: 20 }, (_, i) => i);
  const [selected, setSelected] = [() => 0, (_: number) => {}];

  return (
    <div class="h-20 bg-[#111] border-t border-[#2a2a2a] flex items-center gap-1 px-2 overflow-x-auto flex-shrink-0">
      {photos.map(i => (
        <div
          class={`h-16 w-24 flex-shrink-0 rounded overflow-hidden cursor-pointer relative
            ${i === 0 ? "ring-1 ring-[#4a9eff]" : "hover:ring-1 hover:ring-[#444]"}`}
        >
          <div class="absolute inset-0 bg-gradient-to-br from-[#1e1e1e] to-[#0e0e0e] flex items-center justify-center">
            <span class="text-[#2a2a2a] text-lg">📷</span>
          </div>
          {/* Rating dot */}
          <div class="absolute bottom-0.5 left-0 right-0 flex justify-center gap-0.5">
            {Array.from({ length: 3 }, () => (
              <div class="w-1 h-1 rounded-full bg-[#444]" />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
