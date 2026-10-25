import { useState } from "react";
import { App } from "./App";
import { Td27Panel } from "./Td27Panel";

export function KitControl() {
  const [mode, setMode] = useState<"device" | "offline">("device");
  const [offlineOpened, setOfflineOpened] = useState(false);
  return (
    <>
      <nav className="mode-navigation" aria-label="Editor mode">
        <button
          aria-pressed={mode === "device"}
          onClick={() => setMode("device")}
        >
          TD-27 device
        </button>
        <button
          aria-pressed={mode === "offline"}
          onClick={() => {
            setOfflineOpened(true);
            setMode("offline");
          }}
        >
          Offline workspace
        </button>
      </nav>
      <div hidden={mode !== "device"}>
        <Td27Panel />
      </div>
      {offlineOpened && (
        <div hidden={mode !== "offline"}>
          <App />
        </div>
      )}
    </>
  );
}
