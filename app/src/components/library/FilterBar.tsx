import { createSignal, For } from "solid-js";

export interface FilterState {
  ratingMin: number; // 0-5
  flag: "all" | "pick" | "reject" | "none";
  colorLabel: "all" | "red" | "yellow" | "green" | "blue" | "purple";
  searchQuery: string;
}

interface FilterBarProps {
  filter: FilterState;
  onFilterChange: (f: FilterState) => void;
  totalCount: number;
  filteredCount: number;
}

export function FilterBar(props: FilterBarProps) {
  const handleRatingClick = (rating: number) => {
    props.onFilterChange({
      ...props.filter,
      ratingMin: props.filter.ratingMin === rating ? 0 : rating,
    });
  };

  const handleFlagClick = (flag: "all" | "pick" | "reject" | "none") => {
    props.onFilterChange({
      ...props.filter,
      flag: flag,
    });
  };

  const handleColorClick = (color: "all" | "red" | "yellow" | "green" | "blue" | "purple") => {
    props.onFilterChange({
      ...props.filter,
      colorLabel: color,
    });
  };

  const handleSearchInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    props.onFilterChange({
      ...props.filter,
      searchQuery: target.value,
    });
  };

  const colorLabels = [
    { value: "all" as const, label: "All", color: "#888" },
    { value: "red" as const, label: "Red", color: "#e74c3c" },
    { value: "yellow" as const, label: "Yellow", color: "#f39c12" },
    { value: "green" as const, label: "Green", color: "#27ae60" },
    { value: "blue" as const, label: "Blue", color: "#3498db" },
    { value: "purple" as const, label: "Purple", color: "#9b59b6" },
  ];

  return (
    <div class="border-b border-[#2a2a2a] bg-[#1a1a1a] px-4 py-2">
      <div class="flex items-center gap-4">
        {/* Rating Filter */}
        <div class="flex items-center gap-1">
          <span class="text-xs text-gray-400 mr-2">Rating:</span>
          <For each={[0, 1, 2, 3, 4, 5]}>
            {(rating) => (
              <button
                class={`px-2 py-1 rounded text-xs transition-colors ${
                  props.filter.ratingMin === rating && rating > 0
                    ? "bg-blue-600 text-white"
                    : "bg-[#2a2a2a] text-gray-300 hover:bg-[#3a3a3a]"
                }`}
                onClick={() => handleRatingClick(rating)}
                title={rating === 0 ? "All ratings" : `${rating}+ stars`}
              >
                {rating === 0 ? "All" : "★".repeat(rating)}
              </button>
            )}
          </For>
        </div>

        {/* Flag Filter */}
        <div class="flex items-center gap-1">
          <span class="text-xs text-gray-400 mr-2">Flag:</span>
          <button
            class={`px-3 py-1 rounded text-xs transition-colors ${
              props.filter.flag === "all"
                ? "bg-blue-600 text-white"
                : "bg-[#2a2a2a] text-gray-300 hover:bg-[#3a3a3a]"
            }`}
            onClick={() => handleFlagClick("all")}
          >
            All
          </button>
          <button
            class={`px-3 py-1 rounded text-xs transition-colors ${
              props.filter.flag === "pick"
                ? "bg-green-600 text-white"
                : "bg-[#2a2a2a] text-gray-300 hover:bg-[#3a3a3a]"
            }`}
            onClick={() => handleFlagClick("pick")}
            title="Picks only"
          >
            P
          </button>
          <button
            class={`px-3 py-1 rounded text-xs transition-colors ${
              props.filter.flag === "reject"
                ? "bg-red-600 text-white"
                : "bg-[#2a2a2a] text-gray-300 hover:bg-[#3a3a3a]"
            }`}
            onClick={() => handleFlagClick("reject")}
            title="Rejects only"
          >
            X
          </button>
          <button
            class={`px-3 py-1 rounded text-xs transition-colors ${
              props.filter.flag === "none"
                ? "bg-blue-600 text-white"
                : "bg-[#2a2a2a] text-gray-300 hover:bg-[#3a3a3a]"
            }`}
            onClick={() => handleFlagClick("none")}
            title="Unflagged only"
          >
            U
          </button>
        </div>

        {/* Color Label Filter */}
        <div class="flex items-center gap-1">
          <span class="text-xs text-gray-400 mr-2">Color:</span>
          <For each={colorLabels}>
            {(color) => (
              <button
                class={`w-6 h-6 rounded-full border-2 transition-all ${
                  props.filter.colorLabel === color.value
                    ? "border-white scale-110"
                    : "border-transparent hover:border-gray-500"
                }`}
                style={{ "background-color": color.color }}
                onClick={() => handleColorClick(color.value)}
                title={color.label}
              />
            )}
          </For>
        </div>

        {/* Search Input */}
        <div class="ml-auto flex items-center gap-2">
          <input
            type="text"
            placeholder="Search filename, camera..."
            value={props.filter.searchQuery}
            onInput={handleSearchInput}
            class="px-3 py-1 bg-[#2a2a2a] text-gray-200 text-xs rounded border border-[#3a3a3a] focus:border-blue-500 focus:outline-none w-64"
          />
        </div>
      </div>

      {/* Count Display */}
      <div class="text-xs text-gray-400 mt-2">
        Showing {props.filteredCount} of {props.totalCount} photos
      </div>
    </div>
  );
}
