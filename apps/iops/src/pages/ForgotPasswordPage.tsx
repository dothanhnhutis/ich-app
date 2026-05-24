import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Link } from "react-router";
import appIcon from "@/assets/app-icon.png";

import * as z from "zod";
import { Controller, useForm } from "react-hook-form";
import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";

const forgotPasswordSchema = z.object({
  email: z.email("Email không hợp lệ"),
});

type ForgotPasswordValues = z.infer<typeof forgotPasswordSchema>;

const ForgotPasswordPage = () => {
  const form = useForm<ForgotPasswordValues>({
    resolver: standardSchemaResolver(forgotPasswordSchema),
    defaultValues: {
      email: "gaconght@gmail.com",
    },
  });

  async function onSubmit(data: ForgotPasswordValues) {
    console.log(data);
  }

  return (
    <main className="bg-background p-4 mx-auto max-w-sm flex items-center justify-center">
      <form
        className="flex flex-col gap-6 w-full"
        onSubmit={form.handleSubmit(onSubmit)}
      >
        <FieldGroup>
          <div className="flex flex-col items-center gap-1 text-center">
            <img src={appIcon} alt="I.C.H" className="mb-2 h-20 w-auto" />
          </div>
          <div className="flex flex-col items-center gap-1 text-center">
            <h1 className="text-2xl font-bold">Fotgot your password</h1>
            <p className="text-sm text-balance text-muted-foreground">
              Please enter the email address you'd like your password reset
              information sent to
            </p>
          </div>

          <Controller
            name="email"
            control={form.control}
            render={({ field, fieldState }) => (
              <Field data-invalid={fieldState.invalid}>
                <FieldLabel htmlFor={field.name}>Email</FieldLabel>
                <Input
                  {...field}
                  id={field.name}
                  type="email"
                  aria-invalid={fieldState.invalid}
                  placeholder="m@example.com"
                  required
                  autoComplete="off"
                />
                {fieldState.invalid && (
                  <FieldError errors={[fieldState.error]} />
                )}
              </Field>
            )}
          />

          <Field>
            <Button type="submit">Request reset link</Button>
            <Button variant="link" render={<Link to="/login" />}>
              Back to login
            </Button>
          </Field>
        </FieldGroup>
      </form>
    </main>
  );
};

export default ForgotPasswordPage;
