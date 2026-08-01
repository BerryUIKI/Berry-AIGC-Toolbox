using BerryAIGen.Application.Commands.Albums;
using FluentValidation;

namespace BerryAIGen.Application.Validators.Albums;

public class CreateAlbumCommandValidator : AbstractValidator<CreateAlbumCommand>
{
    public CreateAlbumCommandValidator()
    {
        RuleFor(x => x.Name)
            .NotEmpty().WithMessage("Album name is required")
            .MaximumLength(200).WithMessage("Album name must not exceed 200 characters");
    }
}

public class UpdateAlbumCommandValidator : AbstractValidator<UpdateAlbumCommand>
{
    public UpdateAlbumCommandValidator()
    {
        RuleFor(x => x.Id)
            .NotEmpty().WithMessage("Album ID is required");

        RuleFor(x => x.Name)
            .NotEmpty().WithMessage("Album name is required")
            .MaximumLength(200).WithMessage("Album name must not exceed 200 characters");
    }
}

public class AddImageToAlbumCommandValidator : AbstractValidator<AddImageToAlbumCommand>
{
    public AddImageToAlbumCommandValidator()
    {
        RuleFor(x => x.AlbumId)
            .NotEmpty().WithMessage("Album ID is required");

        RuleFor(x => x.ImageId)
            .NotEmpty().WithMessage("Image ID is required");
    }
}