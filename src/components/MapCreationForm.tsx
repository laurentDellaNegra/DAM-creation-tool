import { useState } from "react";
import { Stack, Flex } from "styled-system/jsx";
import { Button } from "~/components/ui/button";
import { Card } from "~/components/ui/card";
import { Sun, Moon, ListTodoIcon } from "lucide-react";
import { css } from "styled-system/css";
import { IconButton } from "./ui/icon-button";
import BasicSection from "./sections/BasicSection";
import PeriodSection from "./sections/PeriodSection";
import AltitudeCorrectionSection from "./sections/AltitudeCorrectionSection";
import AdditionalInformationSection from "./sections/AdditionalInformationSection";

export default function MapCreationDialog() {
  const [isDarkMode, setIsDarkMode] = useState(false);

  const toggleTheme = () => {
    setIsDarkMode(!isDarkMode);
    document.documentElement.classList.toggle("dark");
  };

  const containerStyle = css({
    minHeight: "1vh",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    padding: "4",
  });

  return (
    <>
      {/* Floating Theme Toggle Button */}
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
        {isDarkMode ? <Sun /> : <Moon />}
      </IconButton>

      <div className={containerStyle}>
        <Card.Root maxWidth="5xl" width="full">
          <Card.Header>
            <Card.Title fontSize="2xl" textAlign="center" color="gray.11">
              DAM Creation Tool
            </Card.Title>
          </Card.Header>

          <Card.Body>
            <Stack gap="6">
              {/* Basic Information Section */}
              <BasicSection />

              {/* Today/Repetitive Periods Section */}
              <PeriodSection />

              {/* Altitude Corrections Section */}
              <AltitudeCorrectionSection />

              {/* Additional Information Section */}
              <AdditionalInformationSection />
            </Stack>
          </Card.Body>

          <Card.Footer gap="3" justifyContent="space-between">
            <Button variant="outline">
              <ListTodoIcon />
              Distribution
            </Button>

            <Flex gap="2">
              <Button>Send</Button>
            </Flex>
          </Card.Footer>
        </Card.Root>
      </div>
    </>
  );
}
