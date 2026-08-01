using BerryAIGen.Application.DTOs;
using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using MediatR;

namespace BerryAIGen.Application.Queries.Albums;

/// <summary>
/// Handler for GetAlbumByIdQuery.
/// </summary>
public class GetAlbumByIdQueryHandler : IRequestHandler<GetAlbumByIdQuery, AlbumDto?>
{
    private readonly IAlbumRepository _albumRepository;

    public GetAlbumByIdQueryHandler(IAlbumRepository albumRepository)
    {
        _albumRepository = albumRepository;
    }

    public async Task<AlbumDto?> Handle(GetAlbumByIdQuery request, CancellationToken cancellationToken)
    {
        var album = await _albumRepository.GetByIdAsync(AlbumId.From(request.Id), cancellationToken);
        if (album == null)
            return null;

        return new AlbumDto
        {
            Id = album.Id,
            Name = album.Name,
            Description = album.Description,
            CoverImageId = album.CoverImageId?.Value,
            ImageCount = album.ImageCount,
            ImageIds = album.ImageIds.Select(id => (Guid)id).ToList(),
            CreatedAt = album.CreatedAt,
            ModifiedAt = album.ModifiedAt
        };
    }
}

/// <summary>
/// Handler for GetAlbumsQuery.
/// </summary>
public class GetAlbumsQueryHandler : IRequestHandler<GetAlbumsQuery, List<AlbumSummaryDto>>
{
    private readonly IAlbumRepository _albumRepository;

    public GetAlbumsQueryHandler(IAlbumRepository albumRepository)
    {
        _albumRepository = albumRepository;
    }

    public async Task<List<AlbumSummaryDto>> Handle(GetAlbumsQuery request, CancellationToken cancellationToken)
    {
        var albums = await _albumRepository.GetAllAsync(cancellationToken);
        return albums.Select(album => new AlbumSummaryDto
        {
            Id = album.Id,
            Name = album.Name,
            Description = album.Description,
            CoverImageId = album.CoverImageId?.Value,
            ImageCount = album.ImageCount,
            CreatedAt = album.CreatedAt
        }).ToList();
    }
}

/// <summary>
/// Handler for GetAlbumsByImageQuery.
/// </summary>
public class GetAlbumsByImageQueryHandler : IRequestHandler<GetAlbumsByImageQuery, List<AlbumSummaryDto>>
{
    private readonly IAlbumRepository _albumRepository;

    public GetAlbumsByImageQueryHandler(IAlbumRepository albumRepository)
    {
        _albumRepository = albumRepository;
    }

    public async Task<List<AlbumSummaryDto>> Handle(GetAlbumsByImageQuery request, CancellationToken cancellationToken)
    {
        var albums = await _albumRepository.GetByImageIdAsync(ImageId.From(request.ImageId), cancellationToken);
        return albums.Select(album => new AlbumSummaryDto
        {
            Id = album.Id,
            Name = album.Name,
            Description = album.Description,
            CoverImageId = album.CoverImageId?.Value,
            ImageCount = album.ImageCount,
            CreatedAt = album.CreatedAt
        }).ToList();
    }
}