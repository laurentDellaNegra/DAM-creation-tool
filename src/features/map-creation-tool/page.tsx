import { Stack, Box } from "styled-system/jsx";
import { Button } from "~/components/ui/button";
import { Card } from "~/components/ui/card";
import { ListTodoIcon } from "lucide-react";
import ThemeSwitcherButton from "~/components/theme-switcher";
import { useAppForm } from "~/hooks/form";
import { damFormOpts } from "./shared-form";
import { damSchema } from "./dam-schemas";
import { GeneralFields } from "./sections/general-fields";
import { PeriodFields } from "./sections/period-fields";
import { AltitudeCorrectionFields } from "./sections/altitude-correction-fields";

export default function MapCreationPage() {
  const form = useAppForm({
    ...damFormOpts,
    validators: {
      onChange: damSchema,
    },
    onSubmit: ({ value }) => {
      alert(JSON.stringify(value, null, 2));
    },
  });
  return (
    <>
      {/* Floating Theme Toggle Button */}
      <ThemeSwitcherButton />

      <form
        onSubmit={(e) => {
          e.preventDefault();
          form.handleSubmit();
        }}
      >
        <Box
          minH="1vh"
          display="flex"
          alignItems="center"
          justifyContent="center"
          p={4}
        >
          <Card.Root maxWidth="5xl" width="full">
            <Card.Header>
              <Card.Title fontSize="2xl" textAlign="center" color="gray.11">
                DAM Creation Tool
              </Card.Title>
            </Card.Header>

            <Card.Body>
              <Stack gap="6">
                {/* Basic Information Section */}
                <GeneralFields form={form} />

                {/* Today/Repetitive Periods Section */}
                <PeriodFields form={form} />

                {/* Altitude Corrections Section */}
                <AltitudeCorrectionFields form={form} />

                {/* Additional Information Section */}
                {/* <AdditionalInformationSection /> */}
              </Stack>
            </Card.Body>

            <Card.Footer gap="3" justifyContent="space-between">
              <Button variant="outline">
                <ListTodoIcon />
                Distribution
              </Button>

              <form.AppForm>
                <form.SubscribeButton label="Send" />
              </form.AppForm>
            </Card.Footer>
          </Card.Root>
        </Box>
      </form>
    </>
  );
}
