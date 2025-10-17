import { useFieldContext } from "~/hooks/form-context";
import { Select } from "../ui/select";
import type { ListCollection } from "@ark-ui/react";
import { CheckIcon, ChevronsUpDownIcon } from "lucide-react";

export type Collection = ListCollection<{
  label: string;
  value: string;
}>;

// TODO: fix types later
// export interface SelectFieldProps extends SelectRootProps<SelectFieldCollection> {
export interface SelectFieldProps {
  label: string;
  collection: Collection;
}

export default function SelectField({ label, collection }: SelectFieldProps) {
  const field = useFieldContext<string>();

  return (
    <Select.Root
      positioning={{ sameWidth: true }}
      collection={collection}
      onValueChange={(e) => field.handleChange(e.value[0])}
      value={field.state.value ? [field.state.value] : []}
    >
      <Select.Label>{label}</Select.Label>
      <Select.Control>
        <Select.Trigger>
          <Select.ValueText placeholder="Select a map" />
          <ChevronsUpDownIcon />
        </Select.Trigger>
      </Select.Control>
      <Select.Positioner>
        <Select.Content>
          <Select.ItemGroup>
            <Select.ItemGroupLabel>Static map</Select.ItemGroupLabel>
            {collection.items.map((item) => (
              <Select.Item key={item.value} item={item}>
                <Select.ItemText>{item.label}</Select.ItemText>
                <Select.ItemIndicator>
                  <CheckIcon />
                </Select.ItemIndicator>
              </Select.Item>
            ))}
          </Select.ItemGroup>
        </Select.Content>
      </Select.Positioner>
    </Select.Root>
  );
}
