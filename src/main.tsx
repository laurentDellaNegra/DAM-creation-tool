import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { TanStackDevtools } from "@tanstack/react-devtools";
import { FormDevtoolsPlugin } from "@tanstack/react-form-devtools";
import { scan } from "react-scan";

scan({
  enabled: true,
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
    <TanStackDevtools
      config={{ hideUntilHover: true }}
      plugins={[FormDevtoolsPlugin()]}
    />
  </StrictMode>
);
