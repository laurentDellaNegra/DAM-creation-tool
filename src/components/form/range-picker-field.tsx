import { useStore } from "@tanstack/react-form";
import { useFieldContext } from "~/hooks/form-context";
import { Field } from "../ui/field";
import { DatePicker } from "../ui/date-picker";
import { parseDate, type DatePickerRootProps } from "@ark-ui/react";
import { Input } from "../ui/input";
import { IconButton } from "../ui/icon-button";
import { CalendarIcon, ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { Button } from "../ui/button";

export interface RangePickerFieldProps extends DatePickerRootProps {
  label: string;
}

export default function RangePickerField({
  label,
  selectionMode = "range",
  positioning = { sameWidth: true },
  startOfWeek = 1,
  ...rest
}: RangePickerFieldProps) {
  const field = useFieldContext<{ startDate: string; endDate: string }>();

  const errors = useStore(field.store, (state) => state.meta.errors);

  const value = [
    ...(field.state.value.startDate
      ? [parseDate(field.state.value.startDate)]
      : []),
    ...(field.state.value.endDate
      ? [parseDate(field.state.value.endDate)]
      : []),
  ];

  return (
    <DatePicker.Root
      positioning={positioning}
      startOfWeek={startOfWeek}
      selectionMode={selectionMode}
      value={value}
      onValueChange={(e) =>
        field.handleChange({
          startDate: e.value?.[0]?.toString() ?? "",
          endDate: e.value?.[1]?.toString() ?? "",
        })
      }
      {...rest}
    >
      <DatePicker.Label>{label}</DatePicker.Label>
      <DatePicker.Control gap={4}>
        <DatePicker.Input index={0} asChild>
          <Input />
        </DatePicker.Input>
        <DatePicker.Input index={1} asChild>
          <Input />
        </DatePicker.Input>
        <DatePicker.Trigger asChild>
          <IconButton variant="outline" aria-label="Open date picker">
            <CalendarIcon />
          </IconButton>
        </DatePicker.Trigger>
      </DatePicker.Control>
      <DatePicker.Positioner>
        <DatePicker.Content>
          <DatePicker.View view="day">
            <DatePicker.Context>
              {(api) => (
                <>
                  <DatePicker.ViewControl>
                    <DatePicker.PrevTrigger asChild>
                      <IconButton variant="ghost" size="sm">
                        <ChevronLeftIcon />
                      </IconButton>
                    </DatePicker.PrevTrigger>
                    <DatePicker.ViewTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <DatePicker.RangeText />
                      </Button>
                    </DatePicker.ViewTrigger>
                    <DatePicker.NextTrigger asChild>
                      <IconButton variant="ghost" size="sm">
                        <ChevronRightIcon />
                      </IconButton>
                    </DatePicker.NextTrigger>
                  </DatePicker.ViewControl>
                  <DatePicker.Table>
                    <DatePicker.TableHead>
                      <DatePicker.TableRow>
                        {api.weekDays.map((weekDay, id) => (
                          <DatePicker.TableHeader key={id}>
                            {weekDay.narrow}
                          </DatePicker.TableHeader>
                        ))}
                      </DatePicker.TableRow>
                    </DatePicker.TableHead>
                    <DatePicker.TableBody>
                      {api.weeks.map((week, id) => (
                        <DatePicker.TableRow key={id}>
                          {week.map((day, id) => (
                            <DatePicker.TableCell key={id} value={day}>
                              <DatePicker.TableCellTrigger asChild>
                                <IconButton variant="ghost">
                                  {day.day}
                                </IconButton>
                              </DatePicker.TableCellTrigger>
                            </DatePicker.TableCell>
                          ))}
                        </DatePicker.TableRow>
                      ))}
                    </DatePicker.TableBody>
                  </DatePicker.Table>
                </>
              )}
            </DatePicker.Context>
          </DatePicker.View>
          <DatePicker.View view="month">
            <DatePicker.Context>
              {(api) => (
                <>
                  <DatePicker.ViewControl>
                    <DatePicker.PrevTrigger asChild>
                      <IconButton variant="ghost" size="sm">
                        <ChevronLeftIcon />
                      </IconButton>
                    </DatePicker.PrevTrigger>
                    <DatePicker.ViewTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <DatePicker.RangeText />
                      </Button>
                    </DatePicker.ViewTrigger>
                    <DatePicker.NextTrigger asChild>
                      <IconButton variant="ghost" size="sm">
                        <ChevronRightIcon />
                      </IconButton>
                    </DatePicker.NextTrigger>
                  </DatePicker.ViewControl>
                  <DatePicker.Table>
                    <DatePicker.TableBody>
                      {api
                        .getMonthsGrid({
                          columns: 4,
                          format: "short",
                        })
                        .map((months, id) => (
                          <DatePicker.TableRow key={id}>
                            {months.map((month, id) => (
                              <DatePicker.TableCell
                                key={id}
                                value={month.value}
                              >
                                <DatePicker.TableCellTrigger asChild>
                                  <Button variant="ghost">{month.label}</Button>
                                </DatePicker.TableCellTrigger>
                              </DatePicker.TableCell>
                            ))}
                          </DatePicker.TableRow>
                        ))}
                    </DatePicker.TableBody>
                  </DatePicker.Table>
                </>
              )}
            </DatePicker.Context>
          </DatePicker.View>
          <DatePicker.View view="year">
            <DatePicker.Context>
              {(api) => (
                <>
                  <DatePicker.ViewControl>
                    <DatePicker.PrevTrigger asChild>
                      <IconButton variant="ghost" size="sm">
                        <ChevronLeftIcon />
                      </IconButton>
                    </DatePicker.PrevTrigger>
                    <DatePicker.ViewTrigger asChild>
                      <Button variant="ghost" size="sm">
                        <DatePicker.RangeText />
                      </Button>
                    </DatePicker.ViewTrigger>
                    <DatePicker.NextTrigger asChild>
                      <IconButton variant="ghost" size="sm">
                        <ChevronRightIcon />
                      </IconButton>
                    </DatePicker.NextTrigger>
                  </DatePicker.ViewControl>
                  <DatePicker.Table>
                    <DatePicker.TableBody>
                      {api.getYearsGrid({ columns: 4 }).map((years, id) => (
                        <DatePicker.TableRow key={id}>
                          {years.map((year, id) => (
                            <DatePicker.TableCell key={id} value={year.value}>
                              <DatePicker.TableCellTrigger asChild>
                                <Button variant="ghost">{year.label}</Button>
                              </DatePicker.TableCellTrigger>
                            </DatePicker.TableCell>
                          ))}
                        </DatePicker.TableRow>
                      ))}
                    </DatePicker.TableBody>
                  </DatePicker.Table>
                </>
              )}
            </DatePicker.Context>
          </DatePicker.View>
        </DatePicker.Content>
      </DatePicker.Positioner>
      {errors.map((error) => (
        <Field.ErrorText key={error}>{error.message}</Field.ErrorText>
      ))}
    </DatePicker.Root>
  );
}
