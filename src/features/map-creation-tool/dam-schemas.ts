import z from "zod";

const swissTimeSchema = z
  .string()
  .regex(
    /^([01]?[0-9]|2[0-3]):([0-5][0-9])$/,
    "Time must be in format HH:MM (00:00-23:59)"
  );

export const damSchema = z.object({
  general: z.object({
    mapName: z.string().max(128, "DAM name too long."),
    rangeDate: z.object({
      startDate: z.string(),
      endDate: z.string(),
    }),
    map: z.string(),
    dl: z.boolean(),
  }),
  periods: z.array(
    z.object({
      startTime: swissTimeSchema,
      endTime: swissTimeSchema,
      lowerLimit: z.number().min(0).max(99999),
      upperLimit: z.number().min(0).max(99999),
      lowerLimitIsFeet: z.boolean(),
      upperLimitIsFeet: z.boolean(),
    })
  ),
  altitudeCorrection: z.object({
    qnhCorr: z.boolean(),
    flCorr: z.boolean(),
    ulHalfBuffer: z.boolean(),
    llHalfBuffer: z.boolean(),
    ulNoBuffer: z.boolean(),
    llNoBuffer: z.boolean(),
  }),
  additionalInformation: z.object({
    text: z.string().max(512, "Text is too long"),
    dabsInfo: z.string().max(512, "DABS info is too long"),
  }),
});
