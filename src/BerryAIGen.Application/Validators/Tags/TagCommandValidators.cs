using BerryAIGen.Application.Commands.Tags;
using FluentValidation;

namespace BerryAIGen.Application.Validators.Tags;

public class CreateTagCommandValidator : AbstractValidator<CreateTagCommand>
{
    public CreateTagCommandValidator()
    {
        RuleFor(x => x.Name)
            .NotEmpty().WithMessage("Tag name is required")
            .MaximumLength(100).WithMessage("Tag name must not exceed 100 characters");

        RuleFor(x => x.Color)
            .Must(BeValidHexColor).WithMessage("Color must be a valid hex color (e.g., #FF0000)")
            .When(x => x.Color != null);

        RuleFor(x => x.Category)
            .MaximumLength(50).WithMessage("Category must not exceed 50 characters")
            .When(x => x.Category != null);
    }

    private static bool BeValidHexColor(string? color)
    {
        return color != null &&
               color.StartsWith('#') &&
               color.Length == 7 &&
               color.Skip(1).All(c => char.IsAsciiHexDigit(c));
    }
}

public class UpdateTagCommandValidator : AbstractValidator<UpdateTagCommand>
{
    public UpdateTagCommandValidator()
    {
        RuleFor(x => x.Id).NotEmpty().WithMessage("Tag ID is required");
        RuleFor(x => x.Name)
            .NotEmpty().WithMessage("Tag name is required")
            .MaximumLength(100).WithMessage("Tag name must not exceed 100 characters");
    }
}