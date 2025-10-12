import { css } from "styled-system/css";
import { IconButton } from "./ui/icon-button";
import { MoonIcon, SunIcon } from "lucide-react";
import { useState } from "react";

export default function ThemeSwitcherButton() {
  const [isDarkMode, setIsDarkMode] = useState(false);

  const toggleTheme = () => {
    setIsDarkMode(!isDarkMode);
    document.documentElement.classList.toggle("dark");
  };
  return (
    <IconButton
      variant="outline"
      rounded="full"
      className={css({
        position: "fixed",
        top: 2,
        right: 2,
        p: 1,
        zIndex: 10,
      })}
      onClick={toggleTheme}
      aria-label={isDarkMode ? "Switch to light mode" : "Switch to dark mode"}
    >
      {isDarkMode ? <SunIcon /> : <MoonIcon />}
    </IconButton>
  );
}
