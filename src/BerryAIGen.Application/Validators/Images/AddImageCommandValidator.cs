using BerryAIGen.Application.Commands.Images;
using FluentValidation;

namespace BerryAIGen.Application.Validators.Images;

/// <summary>
/// Validator for AddImageCommand.
/// </summary>
public class AddImageCommandValidator : AbstractValidator<AddImageCommand>
{
    public AddImageCommandValidator()
    {
        RuleFor(x => x.Path)
            .NotEmpty().WithMessage("Path is required")
            .MaximumLength(2048).WithMessage("Path must not exceed 2048 characters");

        RuleFor(x => x.Hash)
            .NotEmpty().WithMessage("Hash is required")
            .Length(64, 128).WithMessage("Hash must be between 64 and 128 characters");

        RuleFor(x => x.Width)
            .GreaterThan(0).WithMessage("Width must be greater than 0");

        RuleFor(x => x.Height)
            .GreaterThan(0).WithMessage("Height must be greater than 0");

        RuleFor(x => x.FileSize)
            .GreaterThan(0).WithMessage("File size must be greater than 0");

        RuleFor(x => x.FolderId)
            .NotEmpty().WithMessage("Folder ID is required");

        RuleFor(x => x.ImageType)
            .NotEmpty().WithMessage("Image type is required")
            .Must(BeValidImageType).WithMessage("Invalid image type");
    }

    private static bool BeValidImageType(string imageType)
    {
        var validTypes = new[] { "PNG", "JPG", "JPEG", "WEBP", "GIF", "BMP" };
        return validTypes.Contains(imageType.ToUpperInvariant());
    }
}