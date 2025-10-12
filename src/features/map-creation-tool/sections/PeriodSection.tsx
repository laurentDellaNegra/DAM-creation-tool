import { HStack } from "styled-system/jsx";
import { Button } from "~/components/ui/button";
import { Field } from "~/components/ui/field";
import { Checkbox } from "~/components/ui/checkbox";
import { ClockIcon, Plus, Trash } from "lucide-react";
import { css } from "styled-system/css";
import { Fieldset } from "../../../components/ui/fieldset";
import { useState } from "react";

export default function PeriodSection() {
  const [lowerLimitFeet, setLowerLimitFeet] = useState("0");
  const [upperLimitFeet, setUpperLimitFeet] = useState("999");
  return (
    <Fieldset.Root>
      <Fieldset.Legend>
        <HStack>
          <ClockIcon className={css({ w: "4", h: "4" })} />
          Today/Repetitive Periods
        </HStack>
      </Fieldset.Legend>
      <Fieldset.HelperText>
        Configure start and end periods with altitude limits in feet (ft) or
        flight level (FL).
      </Fieldset.HelperText>
      <Fieldset.Control gap={10}>
        <HStack>
          <Field.Root>
            <Field.Label>Start</Field.Label>
            <Field.Input placeholder="09:00" maxLength={5} width={20} />
          </Field.Root>
          <Field.Root>
            <Field.Label>End</Field.Label>
            <Field.Input placeholder="10:00" maxLength={5} width={20} />
          </Field.Root>
          <HStack alignItems="flex-end">
            <Field.Root ml="4">
              <Field.Label>Lower Limit</Field.Label>
              <Field.Input
                value={lowerLimitFeet}
                onChange={(e) => setLowerLimitFeet(e.target.value)}
                maxLength={5}
                width={20}
              />
            </Field.Root>
            <Checkbox>
              <span className={css({ fontSize: "sm" })}>Feet</span>
            </Checkbox>
          </HStack>
          <HStack alignItems="flex-end">
            <Field.Root>
              <Field.Label>Upper Limit</Field.Label>
              <Field.Input
                value={upperLimitFeet}
                onChange={(e) => setUpperLimitFeet(e.target.value)}
                maxLength={5}
                width={20}
              />
            </Field.Root>
            <Checkbox>
              <span className={css({ fontSize: "sm" })}>Feet</span>
            </Checkbox>
          </HStack>
        </HStack>
        <HStack>
          <Button variant="outline" ml="auto">
            <Plus />
            Add Period
          </Button>
          <Button variant="outline">
            <Trash />
            Remove Period
          </Button>
        </HStack>
      </Fieldset.Control>
    </Fieldset.Root>
  );
}
