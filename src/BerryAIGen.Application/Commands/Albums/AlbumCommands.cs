using MediatR;

namespace BerryAIGen.Application.Commands.Albums;

/// <summary>
/// Command to create a new album.
/// </summary>
public record CreateAlbumCommand : IRequest<Guid>
{
    /// <summary>
    /// Gets the album name.
    /// </summary>
    public string Name { get; init; } = string.Empty;

    /// <summary>
    /// Gets the description.
    /// </summary>
    public string? Description { get; init; }
}

/// <summary>
/// Command to update an album.
/// </summary>
public record UpdateAlbumCommand : IRequest<Unit>
{
    /// <summary>
    /// Gets the album ID.
    /// </summary>
    public Guid Id { get; init; }

    /// <summary>
    /// Gets the new name.
    /// </summary>
    public string Name { get; init; } = string.Empty;

    /// <summary>
    /// Gets the new description.
    /// </summary>
    public string? Description { get; init; }
}

/// <summary>
/// Command to delete an album.
/// </summary>
public record DeleteAlbumCommand : IRequest<Unit>
{
    /// <summary>
    /// Gets the album ID.
    /// </summary>
    public Guid Id { get; init; }
}

/// <summary>
/// Command to add an image to an album.
/// </summary>
public record AddImageToAlbumCommand : IRequest<Unit>
{
    /// <summary>
    /// Gets the album ID.
    /// </summary>
    public Guid AlbumId { get; init; }

    /// <summary>
    /// Gets the image ID.
    /// </summary>
    public Guid ImageId { get; init; }
}

/// <summary>
/// Command to remove an image from an album.
/// </summary>
public record RemoveImageFromAlbumCommand : IRequest<Unit>
{
    /// <summary>
    /// Gets the album ID.
    /// </summary>
    public Guid AlbumId { get; init; }

    /// <summary>
    /// Gets the image ID.
    /// </summary>
    public Guid ImageId { get; init; }
}