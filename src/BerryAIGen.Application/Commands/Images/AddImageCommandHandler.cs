using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using MediatR;

namespace BerryAIGen.Application.Commands.Images;

/// <summary>
/// Handler for AddImageCommand.
/// </summary>
public class AddImageCommandHandler : IRequestHandler<AddImageCommand, Guid>
{
    private readonly IImageRepository _imageRepository;
    private readonly IUnitOfWork _unitOfWork;

    /// <summary>
    /// Initializes a new instance of AddImageCommandHandler.
    /// </summary>
    public AddImageCommandHandler(
        IImageRepository imageRepository,
        IUnitOfWork unitOfWork)
    {
        _imageRepository = imageRepository;
        _unitOfWork = unitOfWork;
    }

    /// <inheritdoc/>
    public async Task<Guid> Handle(AddImageCommand request, CancellationToken cancellationToken)
    {
        var image = Domain.Entities.Image.Create(
            FilePath.From(request.Path),
            Hash.From(request.Hash),
            ImageDimensions.From(request.Width, request.Height),
            request.FileSize,
            FolderId.From(request.FolderId),
            request.ImageType);

        await _imageRepository.AddAsync(image, cancellationToken);
        await _unitOfWork.SaveChangesAsync(cancellationToken);

        return image.Id;
    }
}