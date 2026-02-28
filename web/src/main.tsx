import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app";
import { initColorMode } from "./lib/theme";
import "./index.css";

initColorMode();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
