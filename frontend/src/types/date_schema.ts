import { DateTime } from "@/services/date";
import { z } from "zod";

export const date_time_schema = z.custom<Date|string|number>().refine((val) => 
    {
        const date = DateTime.parse(val);
        return !isNaN(date.date.getTime());
    }, 
    {
        message: "Неправильный формат даты"
    })
.transform((val) => DateTime.parse(val));