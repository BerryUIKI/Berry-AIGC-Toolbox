using MediatR;

namespace BerryAIGen.Application.Commands.Images;

/// <summary>
/// Command to add a new image to the library.
/// </summary>
public record AddImageCommand : IRequest<Guid>
{
    /// <summary>
    /// Gets the file path.
    /// </summary>
    public string Path { get; init; } = string.Empty;

    /// <summary>
    /// Gets the file hash.
    /// </summary>
    public string Hash { get; init; } = string.Empty;

    /// <summary>
    /// Gets the width.
    /// </summary>
    public int Width { get; init; }

    /// <summary>
    /// Gets the height.
    /// </summary>
    public int Height { get; init; }

    /// <summary>
    /// Gets the file size.
    /// </summary>
    public long FileSize { get; init; }

    /// <summary>
    /// Gets the folder ID.
    /// </summary>
    public Guid FolderId { get; init; }

    /// <summary>
    /// Gets the image type.
    /// </summary>
    public string ImageType { get; init; } = string.Empty;
}