using BerryAIGen.Domain.Entities;

namespace BerryAIGen.Domain.Interfaces.Repositories;

/// <summary>
/// Repository interface for Tag entity.
/// </summary>
public interface ITagRepository
{
    /// <summary>
    /// Gets a tag by its ID.
    /// </summary>
    Task<Tag?> GetByIdAsync(TagId id, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets a tag by its name.
    /// </summary>
    Task<Tag?> GetByNameAsync(string name, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all tags.
    /// </summary>
    Task<List<Tag>> GetAllAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all tags in a specific category.
    /// </summary>
    Task<List<Tag>> GetByCategoryAsync(string category, CancellationToken cancellationToken = default);

    /// <summary>
    /// Adds a new tag.
    /// </summary>
    Task AddAsync(Tag tag, CancellationToken cancellationToken = default);

    /// <summary>
    /// Updates an existing tag.
    /// </summary>
    void Update(Tag tag);

    /// <summary>
    /// Deletes a tag.
    /// </summary>
    void Delete(Tag tag);

    /// <summary>
    /// Gets the count of all tags.
    /// </summary>
    Task<int> GetCountAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Checks if a tag exists with the given name.
    /// </summary>
    Task<bool> ExistsByNameAsync(string name, CancellationToken cancellationToken = default);
}