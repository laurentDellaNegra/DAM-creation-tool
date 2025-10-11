import { Flex, Grid, HStack } from "styled-system/jsx";
import { Field } from "~/components/ui/field";
import { Checkbox } from "~/components/ui/checkbox";
import { MapIcon } from "lucide-react";
import { css } from "styled-system/css";
import { Fieldset } from "../ui/fieldset";
import { useState } from "react";

export default function AltitudeCorrectionSection() {
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
  return (
    <Fieldset.Root>
      <Fieldset.Legend>
        <HStack>
          <MapIcon className={css({ w: "4", h: "4" })} />
          Altitude Corrections
        </HStack>
      </Fieldset.Legend>
      <Fieldset.HelperText>
        Select altitude correction methods and buffer settings.
      </Fieldset.HelperText>
      <Fieldset.Control>
        <Grid columns={{ base: 2, md: 3 }} gap="4">
          <Field.Root>
            <Flex gap="2" alignItems="center">
              <Checkbox
                checked={altitudeCorrections.qnhCorr}
                onCheckedChange={() =>
                  handleAltitudeCorrectionChange("qnhCorr")
                }
              />
              <Field.Label>QNH-corr</Field.Label>
            </Flex>
            <Field.HelperText>QNH pressure correction</Field.HelperText>
          </Field.Root>
          <Field.Root>
            <Flex gap="2" alignItems="center">
              <Checkbox
                checked={altitudeCorrections.flCorr}
                onCheckedChange={() => handleAltitudeCorrectionChange("flCorr")}
              />
              <Field.Label>FL Corr</Field.Label>
            </Flex>
            <Field.HelperText>Flight level correction</Field.HelperText>
          </Field.Root>
          <Field.Root>
            <Flex gap="2" alignItems="center">
              <Checkbox
                checked={altitudeCorrections.ul12Buffer}
                onCheckedChange={() =>
                  handleAltitudeCorrectionChange("ul12Buffer")
                }
              />
              <Field.Label>UL 1/2 buffer</Field.Label>
            </Flex>
            <Field.HelperText>Upper limit half buffer</Field.HelperText>
          </Field.Root>
          <Field.Root>
            <Flex gap="2" alignItems="center">
              <Checkbox
                checked={altitudeCorrections.ulNoBuffer}
                onCheckedChange={() =>
                  handleAltitudeCorrectionChange("ulNoBuffer")
                }
              />
              <Field.Label>UL No buffer</Field.Label>
            </Flex>
            <Field.HelperText>Upper limit no buffer</Field.HelperText>
          </Field.Root>
          <Field.Root>
            <Flex gap="2" alignItems="center">
              <Checkbox
                checked={altitudeCorrections.ll12Buffer}
                onCheckedChange={() =>
                  handleAltitudeCorrectionChange("ll12Buffer")
                }
              />
              <Field.Label>LL 1/2 buffer</Field.Label>
            </Flex>
            <Field.HelperText>Lower limit half buffer</Field.HelperText>
          </Field.Root>
          <Field.Root>
            <Flex gap="2" alignItems="center">
              <Checkbox
                checked={altitudeCorrections.llNoBuffer}
                onCheckedChange={() =>
                  handleAltitudeCorrectionChange("llNoBuffer")
                }
              />
              <Field.Label>LL No buffer</Field.Label>
            </Flex>
            <Field.HelperText>Lower limit no buffer</Field.HelperText>
          </Field.Root>
        </Grid>
      </Fieldset.Control>
    </Fieldset.Root>
  );
}
