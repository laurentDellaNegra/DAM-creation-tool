import { Field } from "~/components/ui/field";
import { FileTextIcon } from "lucide-react";
import { css } from "styled-system/css";
import { Fieldset } from "../ui/fieldset";
import { Textarea } from "../ui/textarea";
import { HStack } from "styled-system/jsx";

export default function AdditionalInformationSection() {
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
      <Fieldset.Control>
        <HStack gap={4}>
          <Field.Root>
            <Field.Label>Text</Field.Label>
            <Textarea placeholder="Enter text..." rows={4} />
            <Field.HelperText>Additional text information</Field.HelperText>
          </Field.Root>
          <Field.Root>
            <Field.Label>DABS Info</Field.Label>
            <Textarea placeholder="Enter DABS info..." rows={4} />
            <Field.HelperText>DABS system information</Field.HelperText>
          </Field.Root>
        </HStack>
      </Fieldset.Control>
    </Fieldset.Root>
  );
}
