import { HStack } from "styled-system/jsx";
import { FileTextIcon } from "lucide-react";
import { css } from "styled-system/css";
import { Fieldset } from "../../../components/ui/fieldset";
import { withForm } from "~/hooks/form";
import { damFormOpts } from "../shared-form";

export const AdditionalInformationFields = withForm({
  ...damFormOpts,
  render: ({ form }) => {
    return (
      <Fieldset.Root>
        <Fieldset.Legend>
          <HStack>
            <FileTextIcon className={css({ w: "4", h: "4" })} />
            Additional Information
          </HStack>
        </Fieldset.Legend>
        <Fieldset.HelperText>
          Add text descriptions and DABS information for the map.
        </Fieldset.HelperText>
        <Fieldset.Control gap={10}>
          <HStack gap={4}>
            <form.AppField
              name="additionalInformation.text"
              children={(field) => (
                <field.TextAreaField
                  label="Text"
                  placeholder="Enter text..."
                  rows={4}
                  helperText="Additional text information"
                />
              )}
            />

            <form.AppField
              name="additionalInformation.dabsInfo"
              children={(field) => (
                <field.TextAreaField
                  label="DABS Info"
                  placeholder="Enter DABS info..."
                  rows={4}
                  helperText="DABS system information"
                />
              )}
            />
          </HStack>
        </Fieldset.Control>
      </Fieldset.Root>
    );
  },
});
