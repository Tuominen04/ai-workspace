import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import App from "./App";
import { applyTheme, savedTheme } from "./lib/theme";
import "./styles.css";

// Before the first paint, so a chosen theme never flashes the other one.
applyTheme(savedTheme());

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
