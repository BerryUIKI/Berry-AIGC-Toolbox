using BerryAIGen.Application.DTOs;
using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using MediatR;

namespace BerryAIGen.Application.Queries.Images;

/// <summary>
/// Handler for GetImageByIdQuery.
/// </summary>
public class GetImageByIdQueryHandler : IRequestHandler<GetImageByIdQuery, ImageDto?>
{
    private readonly IImageRepository _imageRepository;

    /// <summary>
    /// Initializes a new instance of GetImageByIdQueryHandler.
    /// </summary>
    public GetImageByIdQueryHandler(IImageRepository imageRepository)
    {
        _imageRepository = imageRepository;
    }

    /// <inheritdoc/>
    public async Task<ImageDto?> Handle(GetImageByIdQuery request, CancellationToken cancellationToken)
    {
        var image = await _imageRepository.GetByIdAsync(ImageId.From(request.Id), cancellationToken);
        
        if (image == null)
            return null;

        return new ImageDto
        {
            Id = image.Id,
            Path = image.Path,
            Hash = image.Hash,
            Width = image.Dimensions.Width,
            Height = image.Dimensions.Height,
            FileSize = image.FileSize,
            Rating = image.Rating.Value,
            IsFavorite = image.IsFavorite,
            IsNSFW = image.IsNSFW,
            ImageType = image.ImageType,
            FolderId = image.FolderId,
            CreatedAt = image.CreatedAt,
            ModifiedAt = image.ModifiedAt,
            TagIds = image.TagIds.Select(t => (Guid)t).ToList(),
            AlbumIds = image.AlbumIds.Select(a => (Guid)a).ToList()
        };
    }
}

/// <summary>
/// Handler for GetImagesQuery.
/// </summary>
public class GetImagesQueryHandler : IRequestHandler<GetImagesQuery, List<ImageSummaryDto>>
{
    private readonly IImageRepository _imageRepository;

    /// <summary>
    /// Initializes a new instance of GetImagesQueryHandler.
    /// </summary>
    public GetImagesQueryHandler(IImageRepository imageRepository)
    {
        _imageRepository = imageRepository;
    }

    /// <inheritdoc/>
    public async Task<List<ImageSummaryDto>> Handle(GetImagesQuery request, CancellationToken cancellationToken)
    {
        List<Domain.Entities.Image> images;

        if (request.FolderId.HasValue)
        {
            images = await _imageRepository.GetByFolderIdAsync(
                FolderId.From(request.FolderId.Value), 
                cancellationToken);
        }
        else if (request.FavoritesOnly == true)
        {
            images = await _imageRepository.GetFavoritesAsync(cancellationToken);
        }
        else if (request.NsfwOnly == true)
        {
            images = await _imageRepository.GetNSFWAsync(cancellationToken);
        }
        else
        {
            // For now, return favorites as default set
            // TODO: Implement proper pagination in repository
            images = await _imageRepository.GetFavoritesAsync(cancellationToken);
        }

        return images
            .Skip((request.PageNumber - 1) * request.PageSize)
            .Take(request.PageSize)
            .Select(image => new ImageSummaryDto
            {
                Id = image.Id,
                Path = image.Path,
                Width = image.Dimensions.Width,
                Height = image.Dimensions.Height,
                Rating = image.Rating.Value,
                IsFavorite = image.IsFavorite,
                IsNSFW = image.IsNSFW,
                CreatedAt = image.CreatedAt
            })
            .ToList();
    }
}