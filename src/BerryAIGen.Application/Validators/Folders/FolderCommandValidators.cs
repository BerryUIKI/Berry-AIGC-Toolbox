using BerryAIGen.Application.Commands.Folders;
using FluentValidation;

namespace BerryAIGen.Application.Validators.Folders;

public class CreateFolderCommandValidator : AbstractValidator<CreateFolderCommand>
{
    public CreateFolderCommandValidator()
    {
        RuleFor(x => x.Path)
            .NotEmpty().WithMessage("Folder path is required")
            .MaximumLength(2048).WithMessage("Path must not exceed 2048 characters");
    }
}

public class UpdateFolderCommandValidator : AbstractValidator<UpdateFolderCommand>
{
    public UpdateFolderCommandValidator()
    {
        RuleFor(x => x.Id)
            .NotEmpty().WithMessage("Folder ID is required");
    }
}