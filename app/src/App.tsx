import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { AppShell } from "./components/layout/AppShell";

function App() {
  const [ready, setReady] = createSignal(false);
  const [version, setVersion] = createSignal<any>(null);

  onMount(async () => {
    try {
      const v = await invoke("get_version");
      setVersion(v);
    } catch (e) {
      // dev mode without Tauri — still show the UI
      console.warn("Running without Tauri backend:", e);
    }
    setReady(true);
  });

  return (
    <div class="w-full h-full">
      {ready() && <AppShell version={version()} />}
    </div>
  );
}

export default App;
