import { useState } from "react";
import { Stack, Grid, Flex } from "styled-system/jsx";
import { Button } from "~/components/ui/button";
import { Card } from "~/components/ui/card";
import { Field } from "~/components/ui/field";
import { Checkbox } from "~/components/ui/checkbox";
import { Textarea } from "~/components/ui/textarea";
import { ChevronLeft, ChevronRight, Sun, Moon } from "lucide-react";
import { css } from "styled-system/css";

export default function MapCreationDialog() {
  const [mapName, setMapName] = useState("");
  const [startDate, setStartDate] = useState("2025-10-11");
  const [endDate, setEndDate] = useState("2025-10-11");
  const [lowerLimitFeet, setLowerLimitFeet] = useState("0");
  const [upperLimitFeet, setUpperLimitFeet] = useState("999");
  const [currentPeriod] = useState(1);
  const [totalPeriods] = useState(1);
  const [isDarkMode, setIsDarkMode] = useState(false);

  const [altitudeCorrections, setAltitudeCorrections] = useState({
    qnhCorr: false,
    flCorr: false,
    ul12Buffer: false,
    ulNoBuffer: false,
    ll12Buffer: false,
    llNoBuffer: false,
  });

  const handleAltitudeCorrectionChange = (
    key: keyof typeof altitudeCorrections
  ) => {
    setAltitudeCorrections((prev) => ({
      ...prev,
      [key]: !prev[key],
    }));
  };

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
      <Button
        variant="ghost"
        rounded="full"
        className={css({
          position: "fixed",
          top: 2,
          right: 2,
          p: 1,
        })}
        onClick={toggleTheme}
        aria-label={isDarkMode ? "Switch to light mode" : "Switch to dark mode"}
      >
        {isDarkMode ? <Sun /> : <Moon />}
      </Button>

      <div className={containerStyle}>
        <Card.Root maxWidth="4xl" width="full">
          <Card.Header>
            <Card.Title fontSize="2xl" textAlign="center">
              DAM Creation Tool
            </Card.Title>
          </Card.Header>

          <Card.Body>
            <Stack gap="6">
              {/* Basic Information Section */}
              <Stack gap="4">
                <h3>Basic Information</h3>

                <Grid columns={{ base: 1, md: 3 }} gap="4" alignItems="end">
                  <Field.Root>
                    <Field.Label>Map Name</Field.Label>
                    <Field.Input
                      value={mapName}
                      onChange={(e) => setMapName(e.target.value)}
                      placeholder="Enter map name"
                    />
                  </Field.Root>

                  <Grid columns={2} gap="2">
                    <Field.Root>
                      <Field.Label>Start Date</Field.Label>
                      <Field.Input
                        value={startDate}
                        onChange={(e) => setStartDate(e.target.value)}
                      />
                    </Field.Root>
                    <Field.Root>
                      <Field.Label>End Date</Field.Label>
                      <Field.Input
                        value={endDate}
                        onChange={(e) => setEndDate(e.target.value)}
                      />
                    </Field.Root>
                  </Grid>

                  <Flex gap="2">
                    <Button>Create</Button>
                    <Button variant="outline">Preview</Button>
                    <Button variant="outline" size="sm">
                      DL
                    </Button>
                  </Flex>
                </Grid>
              </Stack>

              {/* Today/Repetitive Periods Section */}
              <Stack gap="4">
                <h3>Today/Repetitive Periods</h3>

                <Grid columns={{ base: 2, md: 4 }} gap="4">
                  <Stack gap="2">
                    <Field.Label>Start</Field.Label>
                    <Checkbox />
                  </Stack>
                  <Stack gap="2">
                    <Field.Label>End</Field.Label>
                    <Checkbox />
                  </Stack>
                  <Stack gap="2">
                    <Field.Label>Lower Limit</Field.Label>
                    <Flex gap="2" alignItems="center">
                      <span className={css({ fontSize: "sm" })}>Feet</span>
                      <Field.Input
                        value={lowerLimitFeet}
                        onChange={(e) => setLowerLimitFeet(e.target.value)}
                        className={css({ w: "20" })}
                      />
                    </Flex>
                  </Stack>
                  <Stack gap="2">
                    <Field.Label>Upper Limit</Field.Label>
                    <Flex gap="2" alignItems="center">
                      <span className={css({ fontSize: "sm" })}>Feet</span>
                      <Field.Input
                        value={upperLimitFeet}
                        onChange={(e) => setUpperLimitFeet(e.target.value)}
                        className={css({ w: "20" })}
                      />
                    </Flex>
                  </Stack>
                </Grid>
              </Stack>

              {/* Period Management Section */}
              <Stack gap="4">
                <h3>Period Management</h3>

                <Flex justify="space-between" alignItems="center">
                  <Flex gap="2">
                    <Button variant="outline">Add Period</Button>
                    <Button variant="outline">Remove Period</Button>
                  </Flex>

                  <Flex gap="2" alignItems="center">
                    <Button variant="outline" size="sm">
                      <ChevronLeft className={css({ w: "4", h: "4" })} />
                    </Button>
                    <span
                      className={css({ fontSize: "sm", fontWeight: "medium" })}
                    >
                      {currentPeriod}/{totalPeriods}
                    </span>
                    <Button variant="outline" size="sm">
                      <ChevronRight className={css({ w: "4", h: "4" })} />
                    </Button>
                  </Flex>
                </Flex>
              </Stack>

              {/* Altitude Corrections Section */}
              <Stack gap="4">
                <h3>Altitude Corrections</h3>

                <Grid columns={{ base: 2, md: 3 }} gap="4">
                  <Flex gap="2" alignItems="center">
                    <Checkbox
                      checked={altitudeCorrections.qnhCorr}
                      onCheckedChange={() =>
                        handleAltitudeCorrectionChange("qnhCorr")
                      }
                    />
                    <Field.Label>QNH-corr</Field.Label>
                  </Flex>
                  <Flex gap="2" alignItems="center">
                    <Checkbox
                      checked={altitudeCorrections.flCorr}
                      onCheckedChange={() =>
                        handleAltitudeCorrectionChange("flCorr")
                      }
                    />
                    <Field.Label>FL Corr</Field.Label>
                  </Flex>
                  <Flex gap="2" alignItems="center">
                    <Checkbox
                      checked={altitudeCorrections.ul12Buffer}
                      onCheckedChange={() =>
                        handleAltitudeCorrectionChange("ul12Buffer")
                      }
                    />
                    <Field.Label>UL 1/2 buffer</Field.Label>
                  </Flex>
                  <Flex gap="2" alignItems="center">
                    <Checkbox
                      checked={altitudeCorrections.ulNoBuffer}
                      onCheckedChange={() =>
                        handleAltitudeCorrectionChange("ulNoBuffer")
                      }
                    />
                    <Field.Label>UL No buffer</Field.Label>
                  </Flex>
                  <Flex gap="2" alignItems="center">
                    <Checkbox
                      checked={altitudeCorrections.ll12Buffer}
                      onCheckedChange={() =>
                        handleAltitudeCorrectionChange("ll12Buffer")
                      }
                    />
                    <Field.Label>LL 1/2 buffer</Field.Label>
                  </Flex>
                  <Flex gap="2" alignItems="center">
                    <Checkbox
                      checked={altitudeCorrections.llNoBuffer}
                      onCheckedChange={() =>
                        handleAltitudeCorrectionChange("llNoBuffer")
                      }
                    />
                    <Field.Label>LL No buffer</Field.Label>
                  </Flex>
                </Grid>
              </Stack>

              {/* Additional Information Section */}
              <Stack gap="4">
                <h3>Additional Information</h3>

                <Grid columns={{ base: 1, md: 2 }} gap="6">
                  <Field.Root>
                    <Field.Label>Text</Field.Label>
                    <Textarea placeholder="Enter text..." rows={4} />
                  </Field.Root>
                  <Field.Root>
                    <Field.Label>DABS Info</Field.Label>
                    <Textarea placeholder="Enter DABS info..." rows={4} />
                  </Field.Root>
                </Grid>
              </Stack>
            </Stack>
          </Card.Body>

          <Card.Footer gap="3" justifyContent="space-between">
            <Button variant="outline">Distribution</Button>

            <Flex gap="2">
              <Button>Send</Button>
              <Button variant="outline">Cancel</Button>
            </Flex>
          </Card.Footer>
        </Card.Root>
      </div>
    </>
  );
}
